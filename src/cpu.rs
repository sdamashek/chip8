use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::{thread, time};
use rand;
use rand::Rng;

use graphics::{Graphics, DrawResult};
use parsing::Instruction;

// A CPUState struct represents the internal state of a Chip8 CPU.
// It includes a Graphics struct implemented in graphics.rs.
#[allow(non_snake_case)]
pub struct CPUState {
    V: [u8; 16],        // General purpose registers: V0, V1, ..., V15
    I: u16,             // Index register 
    pc: u16,            // Program counter (pc)

    memory: [u8; 4096], // 4K memory

    graphics: Graphics, // Graphics struct

    delay_timer: u8,
    sound_timer: u8,    // Sound and delay timers

    stack: [u16; 16],   // Call stack
    sp: u16,            // Call stack pointer
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecResult <'a> {
    Success,
    Fail(&'a str),
    Exit,
}

static CHIP8_FONTSET: [u8; 80] =
[
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl CPUState {
    pub fn new() -> CPUState {
        let mut s = CPUState {
            V: [0; 16],
            I: 0,
            pc: 0x200, // PC starts at 0x200

            memory: [0; 4096],

            graphics: Graphics::new(),

            delay_timer: 0,
            sound_timer: 0, // Initially in-active

            stack: [0; 16],
            sp: 0,
        };

        for i in 0..80 {
            s.memory[i] = CHIP8_FONTSET[i]; // Fill in fontset
        }

        s
    }

    // Load a ROM from fname into memory starting at 0x200.
    pub fn load_rom(&mut self, fname: &str) -> Result<(), &str> {
        let mut f = match File::open(fname) {
            Ok(f) => f,
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => return Err("File not found"),
                    _ => return Err("I/O Error opening"),
                }
            },
        };

        let mut buffer = [0; 128];
        let mut i = 0x200;
        let mut read = 1;

        while read > 0 {
            read = match f.read(&mut buffer) {
                Ok(read) => read,
                Err(_) => return Err("I/O Error reading"),
            };
            if (i + read) > 0xFFF {
                // Too many bytes
                return Err("Too many bytes");
            }
            for j in 0..read {
                self.memory[i + j] = buffer[j];
            }
            i += read;
        }

        Ok(())
    }

    fn valid_addr(&self, addr: u16) -> bool {
        addr <= 0xFFF
    }

    fn valid_pc(&self, addr: u16) -> bool {
        addr >= 0x200 && addr <= 0xFFF
    }

    fn valid_reg(&self, vx: u8) -> bool {
        vx < 16
    }

    // Return the first active key, if any.
    fn active_key(&self, keys: &[bool; 16]) -> Option<u8> {
        for k in 0..16 {
            if keys[k as usize] {
                return Some(k);
            }
        }

        None
    }

    fn clear_op(&mut self) -> ExecResult {
        self.graphics.clear();

        ExecResult::Success
    }

    fn draw_op(&mut self, vx: u8, vy: u8, n: u8) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }
        
        let x = self.V[vx as usize];
        let y = self.V[vy as usize];

        let mem = &self.memory[(self.I as usize)..(self.I as usize + n as usize)];
        match self.graphics.draw_sprite(x, y, mem) {
            DrawResult::Collision => self.V[0xF] = 1,
            DrawResult::Success   => self.V[0xF] = 0,
        };

        ExecResult::Success
    }

    fn return_op(&mut self) -> ExecResult {
        if self.sp == 0 { // Exit
            return ExecResult::Exit;
        }
        
        self.sp -= 1;

        self.pc = self.stack[self.sp as usize];
        if !self.valid_pc(self.pc) {
            return ExecResult::Fail("Invalid return pc");
        }

        ExecResult::Success
    }

    fn jump_op(&mut self, addr: u16) -> ExecResult {
        if !self.valid_pc(addr) {
            return ExecResult::Fail("Invalid jump");
        }

        self.pc = addr;

        ExecResult::Success
    }

    fn call_op(&mut self, addr: u16) -> ExecResult {
        if !self.valid_pc(addr) {
            return ExecResult::Fail("Invalid call");
        }
        if self.sp >= 16 {
            return ExecResult::Fail("Stack overflow");
        }

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        self.pc = addr;

        ExecResult::Success
    }

    fn loadv_op(&mut self, vx: u8, byte: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.V[vx as usize] = byte;

        ExecResult::Success
    }

    fn addv_op(&mut self, vx: u8, byte: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.V[vx as usize] += byte;

        ExecResult::Success
    }

    fn load_op(&mut self, vx: u8, vy: u8) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }

        self.V[vx as usize] = self.V[vy as usize];

        ExecResult::Success
    }

    fn skipv_op(&mut self, vx: u8, byte: u8,
                cond: fn(u8, u8) -> bool) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }
    
        if cond(self.V[vx as usize], byte) {
            self.pc += 2;
        }

        ExecResult::Success
    }

    fn skip_op(&mut self, vx: u8, vy: u8,
               cond: fn(u8, u8) -> bool) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }

        if cond(self.V[vx as usize], self.V[vy as usize]) {
            self.pc += 2;
        }

        ExecResult::Success
    }

    fn arith_op(&mut self, vx: u8, vy: u8,
                arith: fn(u8, u8) -> u8) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }

        let res = arith(self.V[vx as usize], self.V[vy as usize]);
        self.V[vx as usize] = res;

        ExecResult::Success
    }

    fn add_op(&mut self, vx: u8, vy: u8) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }

        let arg1 = self.V[vx as usize];
        let arg2 = self.V[vy as usize];

        let res: u16 = (arg1 as u16) + (arg2 as u16);
        if res > 255 { // Overflow
            self.V[0xF] = 1;
        }
        else {
            self.V[0xF] = 0;
        }

        self.V[vx as usize] = arg1 + arg2;

        ExecResult::Success
    }

    fn sub_op(&mut self, vx: u8, vy: u8) -> ExecResult {
        if !self.valid_reg(vx) || !self.valid_reg(vy) {
            return ExecResult::Fail("Invalid register(s)");
        }

        let arg1 = self.V[vx as usize];
        let arg2 = self.V[vy as usize];

        if arg1 > arg2 { // No carry
            self.V[0xF] = 1;
        }
        else {
            self.V[0xF] = 0;
        }

        self.V[vx as usize] = arg1 - arg2;

        ExecResult::Success
    }

    fn loadi_op(&mut self, addr: u16) -> ExecResult {
        self.I = addr;

        ExecResult::Success
    }

    fn jumpv0_op(&mut self, addr: u16) -> ExecResult {
        let dest = (self.V[0] as u16) + addr;

        if !self.valid_pc(dest) {
            return ExecResult::Fail("Invalid jump destination");
        }

        self.pc = dest;

        ExecResult::Success
    }

    fn rand_op(&mut self, vx: u8, byte: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        let val: u8 = rand::thread_rng().gen();

        self.V[vx as usize] = val & byte;

        ExecResult::Success
    }

    fn skipk_op(&mut self, vx: u8, down: bool) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        let x = self.V[vx as usize];

        if self.graphics.keys[x as usize] == down {
            self.pc += 2;
        }
        
        ExecResult::Success
    }

    fn loaddt_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.V[vx as usize] = self.delay_timer;

        ExecResult::Success
    }

    fn loadwaitk_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        let mut k = self.active_key(&(self.graphics.keys));
        
        while k == None {
            self.graphics.draw_events();
            k = self.active_key(&(self.graphics.keys));
        }

        self.V[vx as usize] = k.unwrap();

        ExecResult::Success
    }

    fn loadtd_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.delay_timer = self.V[vx as usize];

        ExecResult::Success
    }

    fn loadst_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.sound_timer = self.V[vx as usize];

        ExecResult::Success
    }

    fn addi_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.I += self.V[vx as usize] as u16;

        ExecResult::Success
    }

    fn loads_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }

        self.I = 5 * (self.V[vx as usize] as u16);

        ExecResult::Success
    }

    fn loadbcd_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }
        if !self.valid_addr(self.I + 2) {
            return ExecResult::Fail("Invalid destination addr");
        }

        let mut val = self.V[vx as usize];

        let hundreds = val / 100;
        val %= 100;
        let tens = val / 10;
        val %= 10;
        let ones = val;

        self.memory[self.I as usize] = hundreds;
        self.memory[(self.I + 1) as usize] = tens;
        self.memory[(self.I + 2) as usize] = ones;

        ExecResult::Success
    }

    fn loadvm_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }
        if !self.valid_addr(self.I + vx as u16) {
            return ExecResult::Fail("Invalid destination addr");
        }

        for i in 0..(vx + 1) {
            self.memory[(self.I + (i as u16)) as usize] = self.V[i as usize];
        }

        ExecResult::Success
    }

    fn loadmv_op(&mut self, vx: u8) -> ExecResult {
        if !self.valid_reg(vx) {
            return ExecResult::Fail("Invalid register");
        }
        if !self.valid_addr(self.I + vx as u16) {
            return ExecResult::Fail("Invalid destination addr");
        }

        for i in 0..(vx + 1) {
            self.V[i as usize] = self.memory[(self.I + i as u16) as usize];
        }

        ExecResult::Success
    }

    // exec_op executes one CHIP8 instruction.
    // exec_op assumes that PC has already been incremented by 2,
    // and so accordingly PC is the address of the _next_ instruction.
    fn exec_op(&mut self, op: &Instruction) -> ExecResult {
        use Instruction::*;

        match op {
            &Sys(_)     => ExecResult::Success, // We ignore the SYS instruction
            &Cls        => self.clear_op(),
            &Ret        => self.return_op(),
            &Jp(addr)   => self.jump_op(addr),
            &Call(addr) => self.call_op(addr),
            &SeV(vx, byte)  => self.skipv_op(vx, byte, |a, b| a == b),
            &SneV(vx, byte) => self.skipv_op(vx, byte, |a, b| a != b),
            &Se(vx, vy)     => self.skip_op(vx, vy, |a, b| a == b),
            &Sne(vx, vy)    => self.skip_op(vx, vy, |a, b| a != b),
            &LdV(vx, byte)  => self.loadv_op(vx, byte),
            &AddV(vx, byte) => self.addv_op(vx, byte),
            &Ld(vx, vy)     => self.load_op(vx, vy),
            &Or(vx, vy)     => self.arith_op(vx, vy, |a, b| a | b),
            &And(vx, vy)    => self.arith_op(vx, vy, |a, b| a & b),
            &Xor(vx, vy)    => self.arith_op(vx, vy, |a, b| a ^ b),
            &Add(vx, vy)    => self.add_op(vx, vy),
            &Sub(vx, vy)    => self.sub_op(vx, vy),
            &Shr(vx)        => self.arith_op(vx, vx, |a, _| a >> 1),
            &Subn(vx, vy)   => self.sub_op(vy, vx),
            &Shl(vx)        => self.arith_op(vx, vx, |a, _| a << 1),
            &LdI(addr)      => self.loadi_op(addr),
            &JpV0(addr)     => self.jumpv0_op(addr),
            &Rnd(vx, byte)  => self.rand_op(vx, byte),
            &Drw(vx, vy, n) => self.draw_op(vx, vy, n),
            &Skp(vx)        => self.skipk_op(vx, true),
            &Sknp(vx)       => self.skipk_op(vx, false),
            &LdDt(vx)       => self.loaddt_op(vx),
            &LdK(vx)        => self.loadwaitk_op(vx),
            &LdTd(vx)       => self.loadtd_op(vx),
            &LdSt(vx)       => self.loadst_op(vx),
            &AddI(vx)       => self.addi_op(vx),
            &LdS(vx)        => self.loads_op(vx),
            &LdBCD(vx)      => self.loadbcd_op(vx),
            &LdVM(vx)       => self.loadvm_op(vx),
            &LdMV(vx)       => self.loadmv_op(vx),
        }
    }

    // Print registers (for debug purposes)
    fn print_regs(&self) {
        print!("pc = {}, ", self.pc);
        for i in 0..16 {
            print!("V{} = {}, ", i, self.V[i]);
        }
        println!("");
    }

    // Run starting at PC (initially 0x200)
    pub fn run(&mut self) {
        'main: loop {
            self.graphics.draw_events();

            let ins = 
            {
                let memslice = &(self.memory)[(self.pc as usize)..(self.pc as usize + 2)];

                match Instruction::from_slice_one(memslice) {
                    Some(ins) => ins,
                    None => {println!("Invalid instruction {:?}", memslice); break 'main;},
                }
            };

            self.pc += 2;
            
            match self.exec_op(&ins).clone() {
                ExecResult::Fail(e) =>
                    {
                        println!("Error {:?}", e);
                        println!("Instruction: {:?}", &ins);
                        break 'main;
                    },
                ExecResult::Exit => break 'main,
                ExecResult::Success => (),
            }

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.graphics.beep(); // TODO: Implement
                self.sound_timer -= 1;
            }

            thread::sleep(time::Duration::from_millis(5));
        }
    }
}
