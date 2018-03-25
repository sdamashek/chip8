use piston_window::*;
use std::sync::{Arc, Mutex, Condvar};

pub enum GraphicsMessage <'a> {
    Draw {x: u8, y: u8, sprite: &'a [u8]},
    Clear,
}

pub struct Graphics <'a> {
    window: PistonWindow,
    slice: [u8; 64 * 32],
    update: Arc<(Mutex<bool>, Condvar)>,
}

impl<'a> Graphics<'a> {
    pub fn new() -> Graphics<'a> {
        Graphics {
            window: WindowSettings::new(
                        "CHIP8",
                        [64, 32]
                    )
                    .exit_on_esc(true)
                    .build()
                    .unwrap(),
            slice: [0; 64 * 32],
            update: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }


    pub fn draw_loop(&mut self) {
        let lock = self.update.clone();

        while let Some(e) = self.window.next() {
            let &(ref smutex, ref scond) = &*lock;
            let mut updated = smutex.lock().unwrap();
            let result = scond.wait_timeout(updated, Duration::from_millis(10)).unwrap();

            updated = result.0;
            if *updated {
                self.do_update();
            }
        }
    }
}
