use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};
/**
 * Terminal implementation
 */
use std::{io::Stdout, sync::atomic::Ordering};
use std::sync::atomic::AtomicU16;

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

pub struct Vga<'a> {
    width: usize,
    height: usize,
    stdout: Arc<Mutex<Stdout>>,
    vram: &'a Vec<AtomicU16>,
}

impl <'a> Vga<'a> {
    pub fn new(width: usize, height: usize, stdout: Arc<Mutex<Stdout>>, vram: &Vec<AtomicU16>) -> Vga {
        Vga {
            width,
            height,
            stdout,
            vram,
        }
    }

    pub fn reset(&mut self) {
       { 
            let mut stdout = self.stdout.lock().unwrap();

            execute!(stdout, Hide,)
                .expect("Something went wrong writing to the virtual terminal!");

            for a in 0..self.height {
                execute!(
                    stdout,
                    Hide,
                    SetColors(Colors::new(Color::White, Color::Black)),
                    MoveTo(0, a as u16),
                    Print(format!("{: <width$}", "", width = self.width)),
                )
                .expect("Something went wrong writing to the virtual terminal!");
            }
        }

        self.put_diagnostics(0, "Starting ...");
    }

    pub fn put_diagnostics(&mut self, x: u16, s: &str) {
        let mut stdout = self.stdout.lock().unwrap();
        execute!(
            stdout,
            SetColors(Colors::new(Color::White, Color::Blue)),
            MoveTo(x, 0),
            Print(format!("{: <width$}", s, width = self.width)),
        )
        .expect("Something went wrong writing to the virtual terminal!");
    }

    pub fn put_char(&self, stdout: &mut Stdout, offset: usize, val: u16) {
        let x = offset % self.width;
        let y = offset / self.width + 1;

        let bg = ((val & (0b0000011100000000)) >> 8) as usize;
        let fg = ((val & (0b0011100000000000)) >> 11) as usize;

        execute!(
            stdout,
            MoveTo(x as u16, y as u16),
            SetColors(Colors::new(COLORS[fg], COLORS[bg])),
            Print((val & 0x00FF) as u8 as char),
        )
        .expect("Something went wrong writing to the virtual terminal!");
    }

    pub fn start_loop(&mut self) {
        loop {
            for i in 0..self.vram.len() {
                let mut stdout = self.stdout.lock().unwrap();
                self.put_char(&mut stdout, i, self.vram[i].load(Ordering::Relaxed));
            }
        }
    }
}
