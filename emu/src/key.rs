use std::{
    io::stdout,
    process::exit,
    sync::{Arc, Mutex},
};

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::Devices;

pub struct Key {
    irq: Arc<Mutex<bool>>,
    mem: Arc<Mutex<Devices>>,
}

impl Key {
    pub fn new(irq: Arc<Mutex<bool>>, mem: Arc<Mutex<Devices>>) -> Key {
        enable_raw_mode().unwrap();
        Key { irq, mem }
    }

    pub fn handle(&mut self) {
        loop {
            let event = read().unwrap();
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if c >= 'a' && c <= 'z' {
                        *self.irq.lock().unwrap() = true;
                        self.mem.lock().unwrap().key_write(c as u16);
                        // TODO: key_write should take in ps2 code instead
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    execute!(stdout(), ResetColor,).expect("Something went wrong!");
                    disable_raw_mode().unwrap();
                    exit(0);
                }
                _ => (),
            };
        }
    }
}
