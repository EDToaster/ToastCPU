use std::{io::stdout, process::exit, time::Duration};

use crossterm::{
    cursor::Show,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::io::SharedIo;

/// PS/2 set-2 make codes for `a`..=`z`.
const LETTER_CODES: [u16; 26] = [
    0x001C, 0x0032, 0x0021, 0x0023, 0x0024, 0x002B, 0x0034, 0x0033, 0x0043, 0x003B, 0x0042, 0x004B,
    0x003A, 0x0031, 0x0044, 0x004D, 0x0015, 0x002D, 0x001B, 0x002C, 0x003C, 0x002A, 0x001D, 0x0022,
    0x0035, 0x001A,
];

pub const SCAN_ENTER: u16 = 0x005A;

/// Scan code for a lowercase ASCII letter, as produced by the DE1-SoC's PS/2 port.
pub fn letter_scan_code(c: char) -> Option<u16> {
    c.is_ascii_lowercase()
        .then(|| LETTER_CODES[(c as u8 - b'a') as usize])
}

/// Keyboard front-end for terminal mode: reads key events from the tty in raw
/// mode and raises keyboard interrupts on the CPU.
pub struct TerminalKeyboard<'a> {
    io: &'a SharedIo,
}

impl<'a> TerminalKeyboard<'a> {
    pub fn new(io: &'a SharedIo) -> TerminalKeyboard<'a> {
        enable_raw_mode().unwrap();
        TerminalKeyboard { io }
    }

    pub fn handle(&mut self) {
        while !self.io.should_terminate() {
            if !poll(Duration::from_millis(100)).unwrap() {
                continue;
            }

            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let Some(code) = letter_scan_code(c) {
                        self.io.press_key(code);
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }) => self.io.press_key(SCAN_ENTER),
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
