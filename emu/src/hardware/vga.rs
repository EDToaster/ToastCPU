use std::io::Write;
use std::time::{Duration, SystemTime};
use std::{io::Stdout, sync::atomic::Ordering};
use std::sync::atomic::{AtomicU16, AtomicBool, AtomicU64};

use crossterm::QueueableCommand;
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
    stdout: Stdout,
    vram: &'a Vec<AtomicU16>,

    // diagnostics
    interval: Duration,
    prev_time: SystemTime,
    running_count: &'a AtomicU64,
    running_frame_count: usize,
}

impl <'a> Vga<'a> {
    pub fn new(width: usize, height: usize, stdout: Stdout, vram: &'a Vec<AtomicU16>, interval: Duration, running_count: &'a AtomicU64) -> Vga<'a> {
        Vga {
            width,
            height,
            stdout,
            vram,
            interval, 
            prev_time: SystemTime::now(),
            running_count,
            running_frame_count: 0,
        }
    }

    pub fn reset(&mut self) {
       { 
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
        }

        self.put_diagnostics(0, "Starting ...");
    }

    pub fn put_diagnostics(&mut self, x: u16, s: &str) {
        execute!(
            self.stdout,
            SetColors(Colors::new(Color::White, Color::Blue)),
            MoveTo(x, 0),
            Print(format!("{: <width$}", s, width = self.width)),
        ).expect("Something went wrong writing to the virtual terminal!");
    }

    fn char_of(val: u16) -> char {
        match val {
            0x20..=0x7E => val as u8 as char,
            _ => ' '
        }
    }

    pub fn flush_page(&mut self) {
        let mut prevbg = 0; 
        let mut prevfg = 0; 
        let mut first = true;

        for line in 0..60u16 {
            self.stdout.queue(MoveTo(0, line + 1)).unwrap();

            for col in 0..100u16 {
                let offset = line * 100 + col;
                let val = self.vram[offset as usize].load(Ordering::Relaxed);
    
                let bg = ((val & (0b0000011100000000)) >> 8) as usize;
                let fg = ((val & (0b0011100000000000)) >> 11) as usize;
            
                if first || prevbg != bg || prevfg != fg {
                    first = false;
                    prevbg = bg;
                    prevfg = fg;
                    self.stdout.queue(SetColors(Colors::new(COLORS[fg], COLORS[bg]))).unwrap();
                }

                self.stdout.queue(Print(Self::char_of(val & 0x00FF))).unwrap();
            }
        }

        self.stdout.flush().unwrap();

        self.running_frame_count += 1;

        let now: SystemTime = SystemTime::now();

        let duration = now
            .duration_since(self.prev_time)
            .expect("Time went backwards!");

        if duration >= self.interval {
            let insts = self.running_count.swap(0, Ordering::Relaxed);
            let insts_ps = (insts as f64 / duration.as_secs_f64()) as i64;

            let frames = self.running_frame_count;
            let frames_ps = (frames as f64 / duration.as_secs_f64()) as i64;
            self.running_frame_count = 0;

            self.prev_time = now;

            self.put_diagnostics(0, &format!("{insts_ps} ips {frames_ps} fps"));
        }
    }

    pub fn start_loop(&mut self, term: &AtomicBool) {
        while !term.load(Ordering::Relaxed) {
            self.flush_page();
        }
        self.flush_page();
    }
}
