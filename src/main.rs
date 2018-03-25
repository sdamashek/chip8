#[macro_use]
extern crate nom;
extern crate rand;
extern crate piston_window;

mod cpu;
mod parsing;
mod graphics;

use cpu::CPUState;
use parsing::Instruction;

fn main() {
    let mut c = CPUState::new();
    c.load_rom("roms/pong.rom").unwrap();

    return;
}
