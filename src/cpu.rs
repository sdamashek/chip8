use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;

use graphics::Graphics;
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

    key: [bool; 16],    // Key pressed states
}

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

            key: [false; 16],
        };

        for i in 0..80 {
            s.memory[i] = CHIP8_FONTSET[i]; // Fill in fontset
        }

        s
    }

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

    fn valid_pc(&self, addr: u16) -> bool {
        addr >= 0x200 && addr <= 0xFFF && addr % 2 == 0
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
        if !self.valid_addr(addr) {
            return ExecResult::Fail("Invalid jump");
        }

        self.pc = addr;

        ExecResult::Success
    }

    pub fn exec_op(&mut self, op: &Instruction) -> ExecResult {
        use Instruction::*;

        match op {
            &Sys(_) => ExecResult::Success, // We ignore the SYS instruction
            &Ret => self.return_op(),
            _ => ExecResult::Success,
        }
    }


}
