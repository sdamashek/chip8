use std::sync::{Arc, Mutex, Condvar};
use std::process::exit;

use sdl2;
use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::WindowCanvas;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const OUTPUT_WIDTH: u32 = 256;
const OUTPUT_HEIGHT: u32 = 128;

pub struct Graphics {
    context: Sdl,
    canvas: WindowCanvas,
    screen: [bool; 64 * 32],
    
    pub keys: [bool; 16], // Key pressed states
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawResult {
    Collision,
    Success,
}

impl Graphics {
    // Construct a new Graphics struct.
    // Initializes sdl2 and defines an sdl context.
    pub fn new() -> Graphics {
        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys.window("CHIP8", OUTPUT_WIDTH as u32,
                                         OUTPUT_HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Graphics {
            context: sdl_context,
            canvas: canvas,
            screen: [false; 64 * 32],

            keys: [false; 16],
        }
    }

    // Map an SDL Keycode enum to a CHIP8 key, if any.
    fn key_ind(&self, keycode: Keycode) -> Option<usize> {
        match keycode {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q    => Some(0x4),
            Keycode::W    => Some(0x5),
            Keycode::E    => Some(0x6),
            Keycode::R    => Some(0xD),
            Keycode::A    => Some(0x7),
            Keycode::S    => Some(0x8),
            Keycode::D    => Some(0x9),
            Keycode::F    => Some(0xE),
            Keycode::Z    => Some(0xA),
            Keycode::X    => Some(0x0),
            Keycode::C    => Some(0xB),
            Keycode::V    => Some(0xF),
            _             => None,
        }
    }

    // Draw a CHIP8 sprite from a slice to (x, y).
    // If a collision occurs, return Collision. Otherwise, return Success.
    pub fn draw_sprite<'a>(&mut self, x: u8, y: u8, slice: &'a [u8]) -> DrawResult {
        let l = slice.len();
        let mut collision = false;

        for i in 0..l {
            for j in 0..8 {
                let scy = (y as usize + i) % (SCREEN_HEIGHT as usize);
                let scx = (x as usize + j) % (SCREEN_WIDTH as usize);
                
                let scindex = scy * (SCREEN_WIDTH as usize) + scx;
                let set = (slice[i] >> (7 - j)) & 1;
                let set_bool =
                    if set == 1 {
                        true
                    } else {
                        false
                    };

                if self.screen[scindex] && set_bool {
                    collision = true;
                }

                self.screen[scindex] ^= set_bool;
            }
        }
    
        for i in 0..(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize) {
            let cx = (i % (SCREEN_WIDTH as usize)) as i16;
            let cy = (i / (SCREEN_WIDTH as usize)) as i16;
            let mut color = pixels::Color::RGB(0, 0, 0);

            if self.screen[i] {
                color = pixels::Color::RGB(255, 255, 255);
            }

            for j in (cx*4)..(cx*4 + 4) {
                for k in (cy*4)..(cy*4 + 4) {
                    self.canvas.pixel(j, k as i16, color);
                }
            }
        }
        self.canvas.present();

        if collision {
            DrawResult::Collision
        } else {
            DrawResult::Success
        }
    }

    // Clear the canvas.
    pub fn clear(&mut self) {
        for i in 0..self.screen.len() {
            self.screen[i] = false;
        }
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();
    }

    // Process all queued key events.
    pub fn draw_events(&mut self) {
        let mut events = self.context.event_pump().unwrap();

        for event in events.poll_iter() {
            match event {
                Event::Quit {..} => exit(0),

                Event::KeyDown {keycode: Some(keycode), ..} => {
                    if keycode == Keycode::Escape {
                        exit(0);
                    }

                    if let Some(ind) = self.key_ind(keycode) {
                        self.keys[ind as usize] = true;
                    }
                },
                
                Event::KeyUp {keycode: Some(keycode), ..} => {
                    if let Some(ind) = self.key_ind(keycode) {
                        self.keys[ind as usize] = false;
                    }
                },

                _ => {},
            }
        }
    }

    pub fn beep(&mut self) {
        return;
    }
}
