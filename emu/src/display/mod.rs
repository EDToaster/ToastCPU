//! Front-ends that present the VGA text buffer.
//!
//! * [`terminal`] draws the 100x60 character grid into the current terminal
//!   with ANSI escapes (the original behaviour).
//! * [`window`] rasterises the grid with the FPGA's own 6x8 font into a
//!   600x480 framebuffer and shows it in a native window, which is both
//!   pixel-accurate and much cheaper than pushing escape codes through a tty.

pub mod font;
pub mod terminal;
pub mod window;

/// The 3-bit RGB palette used by the VGA driver: bit 2 = red, bit 1 = green,
/// bit 0 = blue, each channel fully on or off.
pub const PALETTE_RGB: [(u8, u8, u8); 8] = [
    (0, 0, 0),
    (0, 0, 255),
    (0, 255, 0),
    (0, 255, 255),
    (255, 0, 0),
    (255, 0, 255),
    (255, 255, 0),
    (255, 255, 255),
];

/// A decoded VRAM cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub ch: u8,
    pub bg: usize,
    pub fg: usize,
}

impl Cell {
    /// Decode a VRAM word: bits 0-7 character, 8-10 background, 11-13 foreground.
    #[inline]
    pub fn decode(val: u16) -> Cell {
        Cell {
            ch: (val & 0x00FF) as u8,
            bg: ((val >> 8) & 0b111) as usize,
            fg: ((val >> 11) & 0b111) as usize,
        }
    }
}

/// Formats the instructions-per-second / frames-per-second readout shared by
/// both front-ends.
pub fn format_rate(insts: u64, frames: usize, secs: f64) -> String {
    let ips = (insts as f64 / secs) as i64;
    let fps = (frames as f64 / secs) as i64;
    format!("{ips} ips {fps} fps")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_cell() {
        // white-on-blue 'A'
        let c = Cell::decode(0b0011_1001_0100_0001);
        assert_eq!(c, Cell { ch: b'A', bg: 1, fg: 7 });
    }
}
