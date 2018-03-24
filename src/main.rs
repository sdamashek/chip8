#[macro_use]
extern crate nom;

mod cpu;
mod parsing;
mod graphics;

use cpu::CPUState;
use parsing::Instruction;

fn main() {
    let ins = Instruction::from_slice(&[0x4B, 0xE0, 0x88, 0x77]);
    println!("Parsed: {:?}", ins);
    return;
}
