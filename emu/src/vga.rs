use std::time::{Duration, SystemTime};
use std::{io::Stdout, sync::atomic::Ordering};
use std::sync::atomic::{AtomicU16, AtomicBool, AtomicU64};

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
        )
        .expect("Something went wrong writing to the virtual terminal!");
    }

    pub fn flush_page(&mut self) {
        for offset in 0..self.vram.len() {
            let val = self.vram[offset].load(Ordering::Relaxed);

            let x = offset % self.width;
            let y = offset / self.width + 1;

            let bg = ((val & (0b0000011100000000)) >> 8) as usize;
            let fg = ((val & (0b0011100000000000)) >> 11) as usize;

            execute!(
                self.stdout,
                MoveTo(x as u16, y as u16),
                SetColors(Colors::new(COLORS[fg], COLORS[bg])),
                Print((val & 0x00FF) as u8 as char),
            )
            .expect("Something went wrong writing to the virtual terminal!");
        }
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
