use std::{
    io::stdout,
    process::exit,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
};

use crossterm::{
    cursor::Show,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode},
};

pub struct Key {
    irq: Arc<Mutex<bool>>,
    key: Arc<Mutex<u16>>,
}

const CODES: [u16; 26] = [
    0x001C, 0x0032, 0x0021, 0x0023, 0x0024, 0x002B, 0x0034, 0x0033, 0x0043, 0x003B, 0x0042, 0x004B,
    0x003A, 0x0031, 0x0044, 0x004D, 0x0015, 0x002D, 0x001B, 0x002C, 0x003C, 0x002A, 0x001D, 0x0022,
    0x0035, 0x001A,
];

impl Key {
    fn convert_to_scan_code(&self, c: u8) -> u16 {
        CODES[(c - b'a') as usize]
    }

    pub fn new(irq: Arc<Mutex<bool>>, key: Arc<Mutex<u16>>) -> Key {
        enable_raw_mode().unwrap();
        Key { irq, key }
    }

    fn irq(&mut self, code: u16) {
        *self.irq.lock().unwrap() = true;
        *self.key.lock().unwrap() = code;
    }

    pub fn handle(&mut self, term: &AtomicBool) {
        while !term.load(Ordering::Relaxed) {
            let event = read().unwrap();
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if c.is_ascii_lowercase() {
                        self.irq(self.convert_to_scan_code(c as u8));
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    self.irq(0x005A);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    execute!(stdout(), ResetColor, Show).expect("Something went wrong!");
                    disable_raw_mode().unwrap();
                    exit(0);
                }
                _ => (),
            };
        }
    }
}
