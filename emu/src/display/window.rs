use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window};

use super::font::{GLYPHS, GLYPH_HEIGHT, GLYPH_WIDTH};
use super::{format_rate, Cell, PALETTE_RGB};
use crate::hardware::def::{VGA_HEIGHT, VGA_WIDTH};
use crate::hardware::key::{letter_scan_code, SCAN_ENTER};
use crate::io::SharedIo;

/// Framebuffer size: 100x60 cells of 6x8 pixels, i.e. the 600x480 active text
/// area of the FPGA's 640x480 VGA mode.
pub const FB_WIDTH: usize = VGA_WIDTH * GLYPH_WIDTH;
pub const FB_HEIGHT: usize = VGA_HEIGHT * GLYPH_HEIGHT;

const TITLE: &str = "ToastCPU";

#[derive(Clone, Copy, Debug)]
pub struct WindowOptions {
    /// Integer upscale applied to the 600x480 framebuffer (1, 2, 4 or 8).
    pub scale: usize,
    /// Present at most this many frames per second (0 = uncapped).
    pub fps: usize,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions { scale: 2, fps: 60 }
    }
}

fn minifb_scale(scale: usize) -> Result<Scale, String> {
    Ok(match scale {
        1 => Scale::X1,
        2 => Scale::X2,
        4 => Scale::X4,
        8 => Scale::X8,
        other => return Err(format!("unsupported window scale {other}; use 1, 2, 4 or 8")),
    })
}

/// Map a window key press to the scan code the PS/2 driver would deliver.
/// Mirrors the terminal front-end: lowercase letters and Enter.
fn scan_code_of(key: Key) -> Option<u16> {
    use Key::*;
    let letter = match key {
        A => 'a', B => 'b', C => 'c', D => 'd', E => 'e', F => 'f', G => 'g',
        H => 'h', I => 'i', J => 'j', K => 'k', L => 'l', M => 'm', N => 'n',
        O => 'o', P => 'p', Q => 'q', R => 'r', S => 's', T => 't', U => 'u',
        V => 'v', W => 'w', X => 'x', Y => 'y', Z => 'z',
        Enter | NumPadEnter => return Some(SCAN_ENTER),
        _ => return None,
    };
    letter_scan_code(letter)
}

/// Rasterises a VRAM cell into a 0RGB framebuffer exactly like the VGA
/// driver's pixel pipeline: glyph bit set -> foreground, clear -> background.
pub fn draw_cell(fb: &mut [u32], palette: &[u32; 8], col: usize, line: usize, val: u16) {
    let cell = Cell::decode(val);
    let glyph = GLYPHS[cell.ch as usize];
    let (fg, bg) = (palette[cell.fg], palette[cell.bg]);

    let mut bit = GLYPH_WIDTH * GLYPH_HEIGHT;
    for y in 0..GLYPH_HEIGHT {
        let row = (line * GLYPH_HEIGHT + y) * FB_WIDTH + col * GLYPH_WIDTH;
        for px in &mut fb[row..row + GLYPH_WIDTH] {
            bit -= 1;
            *px = if (glyph >> bit) & 1 != 0 { fg } else { bg };
        }
    }
}

/// Native-window front-end. Must be driven from the main thread.
pub struct WindowDisplay<'a> {
    io: &'a SharedIo,
    window: Window,
    framebuffer: Vec<u32>,
    palette: [u32; 8],
    // Last-rasterised value of each cell; only changed cells are redrawn.
    // All-zero matches the all-black initial framebuffer.
    shadow: Vec<u16>,

    interval: Duration,
    prev_time: Instant,
    frames: usize,
    rate: String,
}

impl<'a> WindowDisplay<'a> {
    pub fn new(io: &'a SharedIo, opts: WindowOptions) -> Result<WindowDisplay<'a>, String> {
        let mut window = Window::new(
            TITLE,
            FB_WIDTH,
            FB_HEIGHT,
            minifb::WindowOptions {
                scale: minifb_scale(opts.scale)?,
                scale_mode: ScaleMode::AspectRatioStretch,
                resize: true,
                ..minifb::WindowOptions::default()
            },
        )
        .map_err(|e| format!("could not open window: {e}"))?;
        window.set_target_fps(opts.fps);

        Ok(WindowDisplay {
            io,
            window,
            framebuffer: vec![0u32; FB_WIDTH * FB_HEIGHT],
            palette: PALETTE_RGB.map(|(r, g, b)| (r as u32) << 16 | (g as u32) << 8 | b as u32),
            shadow: vec![0u16; VGA_WIDTH * VGA_HEIGHT],
            interval: Duration::from_millis(500),
            prev_time: Instant::now(),
            frames: 0,
            rate: String::new(),
        })
    }

    /// Re-rasterise cells that changed since the last frame. Returns whether
    /// anything changed.
    fn refresh(&mut self) -> bool {
        let mut dirty = false;
        for (i, (cell, seen)) in self.io.vram.iter().zip(self.shadow.iter_mut()).enumerate() {
            let val = cell.load(Ordering::Relaxed);
            if val != *seen {
                *seen = val;
                dirty = true;
                draw_cell(&mut self.framebuffer, &self.palette, i % VGA_WIDTH, i / VGA_WIDTH, val);
            }
        }
        dirty
    }

    fn update_title(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.prev_time);
        if elapsed < self.interval {
            return;
        }
        let insts = self.io.running_count.swap(0, Ordering::Relaxed);
        let rate = format_rate(insts, self.frames, elapsed.as_secs_f64());
        self.frames = 0;
        self.prev_time = now;

        let status = if let Some(err) = self.io.error.get() {
            format!("{TITLE} - stopped: {err}")
        } else if self.io.is_halted() {
            format!("{TITLE} - halted")
        } else {
            format!("{TITLE} - {rate}")
        };
        if status != self.rate {
            self.window.set_title(&status);
            self.rate = status;
        }
    }

    /// Pump the window until it is closed. Keeps showing the final screen
    /// after the program halts.
    pub fn run(&mut self) {
        while self.window.is_open() && !self.io.should_terminate() {
            for key in self.window.get_keys_pressed(KeyRepeat::Yes) {
                if let Some(code) = scan_code_of(key) {
                    self.io.press_key(code);
                }
            }

            // `update*` also paces the loop to the target FPS.
            if self.refresh() {
                self.window
                    .update_with_buffer(&self.framebuffer, FB_WIDTH, FB_HEIGHT)
                    .expect("failed to present framebuffer");
            } else {
                self.window.update();
            }
            self.frames += 1;
            self.update_title();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_glyph_with_fpga_bit_order() {
        let palette = PALETTE_RGB.map(|(r, g, b)| (r as u32) << 16 | (g as u32) << 8 | b as u32);
        let mut fb = vec![0u32; FB_WIDTH * FB_HEIGHT];
        // white 'I' on red at (1, 0). Glyph 0x49 = 0x708208208700:
        // row 0 = 011100, row 1 = 001000, ..., row 7 = 000000.
        draw_cell(&mut fb, &palette, 1, 0, 0b0011_1100_0000_0000 | b'I' as u16);
        let px = |x: usize, y: usize| fb[y * FB_WIDTH + 6 + x];
        let (white, red) = (0xFFFFFF, 0xFF0000);
        assert_eq!((0..6).map(|x| px(x, 0)).collect::<Vec<_>>(), [red, white, white, white, red, red]);
        assert_eq!((0..6).map(|x| px(x, 1)).collect::<Vec<_>>(), [red, red, white, red, red, red]);
        assert!((0..6).all(|x| px(x, 7) == red));
        // neighbouring cell untouched
        assert_eq!(fb[0], 0);
    }

    #[test]
    fn maps_keys_to_scan_codes() {
        assert_eq!(scan_code_of(Key::A), Some(0x1C));
        assert_eq!(scan_code_of(Key::Z), Some(0x1A));
        assert_eq!(scan_code_of(Key::Enter), Some(0x5A));
        assert_eq!(scan_code_of(Key::Space), None);
    }
}
