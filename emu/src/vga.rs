/**
 * Terminal implementation
 */
use std::io::Stdout;

use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    style::{Color, Colors, Print, SetColors},
};

const COLORS: [Color; 8] = [
    Color::Rgb { r: 0, g: 0, b: 0 },
    Color::Rgb { r: 0, g: 0, b: 255 },
    Color::Rgb { r: 0, g: 255, b: 0 },
    Color::Rgb {
        r: 0,
        g: 255,
        b: 255,
    },
    Color::Rgb { r: 255, g: 0, b: 0 },
    Color::Rgb {
        r: 255,
        g: 0,
        b: 255,
    },
    Color::Rgb {
        r: 255,
        g: 255,
        b: 0,
    },
    Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
];

pub struct Vga {
    width: usize,
    height: usize,
    buffer: Vec<u16>,
    stdout: Stdout,

    pub write_count: usize,
}

impl Vga {
    pub fn new(width: usize, height: usize, stdout: Stdout) -> Vga {
        Vga {
            width,
            height,
            buffer: vec![0; width * height],
            stdout,
            write_count: 0,
        }
    }

    pub fn reset(&mut self) {
        execute!(self.stdout, Hide,)
            .expect("Something went wrong writing to the virtual terminal!");

        for a in 0..self.height {
            execute!(
                self.stdout,
                Hide,
                SetColors(Colors::new(Color::White, Color::Black)),
                MoveTo(0, a as u16),
                Print(format!("{: <width$}", "", width = self.width)),
            )
            .expect("Something went wrong writing to the virtual terminal!");
        }

        self.put_diagnostics(0, "Starting ...");
    }

    pub fn put_diagnostics(&mut self, x: u16, s: &str) {
        execute!(
            self.stdout,
            SetColors(Colors::new(Color::White, Color::Blue)),
            MoveTo(x, 0),
            Print(format!("{: <width$}", s, width = self.width)),
        )
        .expect("Something went wrong writing to the virtual terminal!");
    }

    pub fn put_char(&mut self, offset: usize, val: u16) {
        let x = offset % self.width;
        let y = offset / self.width + 1;

        let bg = ((val & (0b0000011100000000)) >> 8) as usize;
        let fg = ((val & (0b0011100000000000)) >> 11) as usize;

        execute!(
            self.stdout,
            SetColors(Colors::new(COLORS[fg as usize], COLORS[bg as usize])),
            MoveTo(x as u16, y as u16),
            Print((val & 0x00FF) as u8 as char),
        )
        .expect("Something went wrong writing to the virtual terminal!");

        self.buffer[offset] = val;
        self.write_count += 1;
    }
}
