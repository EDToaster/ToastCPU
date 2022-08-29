/**
 * Terminal implementation
 */
use std::io::Stdout;

use crossterm::{execute, style::{Color, SetColors, Colors, Print}, cursor::MoveTo};

const COLORS: [Color; 8] = [
    Color::Rgb { r: 0, g: 0, b: 0 },
    Color::Rgb { r: 0, g: 0, b: 255 },
    Color::Rgb { r: 0, g: 255, b: 0 },
    Color::Rgb { r: 0, g: 255, b: 255 },
    Color::Rgb { r: 255, g: 0, b: 0 },
    Color::Rgb { r: 255, g: 0, b: 255 },
    Color::Rgb { r: 255, g: 255, b: 0 },
    Color::Rgb { r: 255, g: 255, b: 255 },
];

pub struct VGA {
    width: usize,
    height: usize,
    buffer: Vec<u16>,
    stdout: Stdout,
}

impl VGA {
    pub fn new(width: usize, height: usize, stdout: Stdout) -> VGA {
        return VGA {
            width, height, 
            buffer: vec![0; width*height],
            stdout,
        }
    }

    pub fn put_char(&mut self, offset: usize, val: u16) {
        let x = offset % self.width;
        let y = offset / self.width;

        let bg = ((val & (0b0000011100000000)) >> 8) as usize;
        let fg = ((val & (0b0011100000000000)) >> 11) as usize;

        execute!(
            self.stdout,
            SetColors(Colors::new(COLORS[fg as usize], COLORS[bg as usize])),
            MoveTo(x as u16, y as u16),
            Print((val & 0x00FF) as u8 as char),
        ).expect("Something went wrong writing to the virtual terminal!");

        self.buffer[offset] = val;
    }
}