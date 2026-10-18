use std::io::{Stdout, Write};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crossterm::QueueableCommand;
use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    style::{Color, Colors, Print, SetColors},
};

use super::{format_rate, Cell, PALETTE_RGB};
use crate::io::SharedIo;

fn color(idx: usize) -> Color {
    let (r, g, b) = PALETTE_RGB[idx];
    Color::Rgb { r, g, b }
}

/// Renders the text buffer into the current terminal using ANSI escapes.
/// Row 0 of the terminal is a diagnostics bar; VGA row N is terminal row N+1.
pub struct TerminalDisplay<'a> {
    width: usize,
    height: usize,
    stdout: Stdout,
    io: &'a SharedIo,
    // Last-rendered copy of each VRAM cell, used for dirty tracking so a frame
    // only re-emits the cells whose value actually changed. A cell of 0 renders
    // as a black-on-black space, which matches the blanked screen after reset(),
    // so an all-zero shadow correctly represents a freshly cleared display.
    shadow: Vec<u16>,

    // diagnostics
    interval: Duration,
    prev_time: Instant,
    running_frame_count: usize,
}

impl<'a> TerminalDisplay<'a> {
    pub fn new(
        width: usize,
        height: usize,
        stdout: Stdout,
        io: &'a SharedIo,
        interval: Duration,
    ) -> TerminalDisplay<'a> {
        TerminalDisplay {
            width,
            height,
            stdout,
            io,
            shadow: vec![0u16; width * height],
            interval,
            prev_time: Instant::now(),
            running_frame_count: 0,
        }
    }

    pub fn reset(&mut self) {
        // The screen is blanked below; keep the shadow in sync so the next
        // frame redraws exactly the non-blank cells.
        self.shadow.iter_mut().for_each(|c| *c = 0);
        execute!(self.stdout, Hide).expect("Something went wrong writing to the virtual terminal!");

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

    fn char_of(ch: u8) -> char {
        match ch {
            0x20..=0x7E => ch as char,
            _ => ' ',
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
                let val = self.io.vram[offset].load(Ordering::Relaxed);

                // Skip cells unchanged since the last frame — this is the win.
                if val == self.shadow[offset] {
                    continue;
                }
                self.shadow[offset] = val;
                let cell = Cell::decode(val);

                // Only reposition the cursor when it isn't already sitting where
                // this cell needs to be printed (after a Print it auto-advances
                // one column, so runs of adjacent dirty cells cost no MoveTo).
                if pen_line != line || pen_col != col {
                    self.stdout.queue(MoveTo(col as u16, (line + 1) as u16)).unwrap();
                }

                if pen_bg != cell.bg || pen_fg != cell.fg {
                    pen_bg = cell.bg;
                    pen_fg = cell.fg;
                    self.stdout
                        .queue(SetColors(Colors::new(color(cell.fg), color(cell.bg))))
                        .unwrap();
                }

                self.stdout.queue(Print(Self::char_of(cell.ch))).unwrap();
                pen_line = line;
                pen_col = col + 1;
            }
        }

        self.stdout.flush().unwrap();

        self.running_frame_count += 1;

        let now = Instant::now();
        let duration = now.duration_since(self.prev_time);

        if duration >= self.interval {
            let insts = self.io.running_count.swap(0, Ordering::Relaxed);
            let frames = self.running_frame_count;
            self.running_frame_count = 0;
            self.prev_time = now;

            self.put_diagnostics(0, &format_rate(insts, frames, duration.as_secs_f64()));
        }
    }

    pub fn start_loop(&mut self) {
        while !self.io.should_terminate() {
            self.flush_page();
        }
        self.flush_page();
    }
}
