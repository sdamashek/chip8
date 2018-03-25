#[macro_use]
extern crate nom;
extern crate rand;
extern crate sdl2;

mod cpu;
mod parsing;
mod graphics;

use cpu::CPUState;
use parsing::Instruction;
use std::thread;
use graphics::Graphics;
use std::sync::Arc;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut c = CPUState::new();
    c.load_rom(&args[1]).unwrap();

    c.run();

    return;
}
