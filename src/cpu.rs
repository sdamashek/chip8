use graphics::Graphics;

// A CPUState struct represents the internal state of a Chip8 CPU.
// It includes a Graphics struct implemented in graphics.rs.
#[derive(Debug)]
struct CPUState {
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

static chip8_fontset: [u8; 80] =
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
    fn new() -> CPUState {
        let s = CPUState {
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

        for i in (0..80) {
            s.memory[i] = chip8_fontset[i]; // Fill in fontset
        }

        s
    }
}
