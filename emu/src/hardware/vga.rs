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
    // Last-rendered copy of each VRAM cell, used for dirty tracking so a frame
    // only re-emits the cells whose value actually changed. A cell of 0 renders
    // as a black-on-black space, which matches the blanked screen after reset(),
    // so an all-zero shadow correctly represents a freshly cleared display.
    shadow: Vec<u16>,

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
            shadow: vec![0u16; width * height],
            interval,
            prev_time: SystemTime::now(),
            running_count,
            running_frame_count: 0,
        }
    }

    pub fn reset(&mut self) {
        // The screen is blanked below; keep the shadow in sync so the next
        // frame redraws exactly the non-blank cells.
        self.shadow.iter_mut().for_each(|c| *c = 0);
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
        // Sentinels (usize::MAX) that never equal a real value, so the first
        // dirty cell of each frame forces both a cursor MoveTo and a SetColors.
        // The terminal's cursor and color state are unknown at the start of a
        // frame (the diagnostics line moves the cursor and changes colors), so
        // we can't carry pen/color state across frames.
        let mut pen_line = usize::MAX;
        let mut pen_col = usize::MAX;
        let mut pen_fg = usize::MAX;
        let mut pen_bg = usize::MAX;

        for line in 0..self.height {
            for col in 0..self.width {
                let offset = line * self.width + col;
                let val = self.vram[offset].load(Ordering::Relaxed);

                // Skip cells unchanged since the last frame — this is the win.
                if val == self.shadow[offset] {
                    continue;
                }
                self.shadow[offset] = val;

                // Only reposition the cursor when it isn't already sitting where
                // this cell needs to be printed (after a Print it auto-advances
                // one column, so runs of adjacent dirty cells cost no MoveTo).
                if pen_line != line || pen_col != col {
                    self.stdout
                        .queue(MoveTo(col as u16, (line + 1) as u16))
                        .unwrap();
                }

                let bg = ((val & 0b0000_0111_0000_0000) >> 8) as usize;
                let fg = ((val & 0b0011_1000_0000_0000) >> 11) as usize;
                if pen_bg != bg || pen_fg != fg {
                    pen_bg = bg;
                    pen_fg = fg;
                    self.stdout
                        .queue(SetColors(Colors::new(COLORS[fg], COLORS[bg])))
                        .unwrap();
                }

                self.stdout.queue(Print(Self::char_of(val & 0x00FF))).unwrap();
                pen_line = line;
                pen_col = col + 1;
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
