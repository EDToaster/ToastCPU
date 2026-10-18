//! Native-window front-end (egui/eframe): the VGA screen, drawn with the
//! FPGA's own font, plus an optional debugger sidebar.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Color32, ColorImage, Event, Key, Modifiers, Panel, Pos2, Rect, Sense, TextureHandle,
    TextureOptions, Vec2,
};

use super::debug_panel::DebugPanel;
use super::font::{GLYPHS, GLYPH_HEIGHT, GLYPH_WIDTH};
use super::{format_rate, Cell, PALETTE_RGB};
use crate::emulator::debugger::{Command, DebugLink, RunState};
use crate::hardware::def::{VGA_HEIGHT, VGA_WIDTH};
use crate::hardware::key::{letter_scan_code, SCAN_ENTER};
use crate::io::SharedIo;
use crate::program::DebugInfo;

/// Framebuffer size: 100x60 cells of 6x8 pixels, i.e. the 600x480 active text
/// area of the FPGA's 640x480 VGA mode.
pub const FB_WIDTH: usize = VGA_WIDTH * GLYPH_WIDTH;
pub const FB_HEIGHT: usize = VGA_HEIGHT * GLYPH_HEIGHT;

const TITLE: &str = "ToastCPU";
const SIDEBAR_WIDTH: f32 = 440.0;
const COLLAPSED_WIDTH: f32 = 26.0;

#[derive(Clone, Copy, Debug)]
pub struct WindowOptions {
    /// Initial integer upscale of the 600x480 screen.
    pub scale: usize,
    /// Present at most this many frames per second (0 = as fast as vsync allows).
    pub fps: usize,
    /// Open with the debugger sidebar expanded.
    pub debug: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions { scale: 2, fps: 60, debug: false }
    }
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
        Enter => return Some(SCAN_ENTER),
        _ => return None,
    };
    letter_scan_code(letter)
}

/// Rasterises a VRAM cell into a framebuffer exactly like the VGA driver's
/// pixel pipeline: glyph bit set -> foreground, clear -> background.
pub fn draw_cell<P: Copy>(fb: &mut [P], palette: &[P; 8], col: usize, line: usize, val: u16) {
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

pub struct WindowApp<'a> {
    io: &'a SharedIo,
    link: &'a DebugLink,
    opts: WindowOptions,

    screen: ColorImage,
    palette: [Color32; 8],
    // Last-rasterised value of each cell; only changed cells are redrawn.
    shadow: Vec<u16>,
    texture: Option<TextureHandle>,

    sidebar_open: bool,
    panel: DebugPanel<'a>,

    // stats
    stats_since: Instant,
    frames: usize,
    rate: String,
    title: String,
}

impl<'a> WindowApp<'a> {
    pub fn new(
        io: &'a SharedIo,
        link: &'a DebugLink,
        rom: &'a [u16],
        debug: &'a DebugInfo,
        opts: WindowOptions,
    ) -> WindowApp<'a> {
        let palette = PALETTE_RGB.map(|(r, g, b)| Color32::from_rgb(r, g, b));
        WindowApp {
            io,
            link,
            opts,
            screen: ColorImage::new([FB_WIDTH, FB_HEIGHT], vec![palette[0]; FB_WIDTH * FB_HEIGHT]),
            palette,
            shadow: vec![0u16; VGA_WIDTH * VGA_HEIGHT],
            texture: None,
            sidebar_open: opts.debug,
            panel: DebugPanel::new(io, link, rom, debug),
            stats_since: Instant::now(),
            frames: 0,
            rate: String::new(),
            title: String::new(),
        }
    }

    /// Initial window size for the given options.
    pub fn initial_size(opts: &WindowOptions) -> Vec2 {
        let side = if opts.debug { SIDEBAR_WIDTH } else { COLLAPSED_WIDTH };
        Vec2::new((FB_WIDTH * opts.scale) as f32 + side, (FB_HEIGHT * opts.scale) as f32)
    }

    /// Re-rasterise cells that changed since the last frame.
    fn refresh_screen(&mut self, ctx: &egui::Context) {
        let mut dirty = false;
        for (i, (cell, seen)) in self.io.vram.iter().zip(self.shadow.iter_mut()).enumerate() {
            let val = cell.load(Ordering::Relaxed);
            if val != *seen {
                *seen = val;
                dirty = true;
                draw_cell(&mut self.screen.pixels, &self.palette, i % VGA_WIDTH, i / VGA_WIDTH, val);
            }
        }
        match &mut self.texture {
            None => {
                self.texture = Some(ctx.load_texture("vga", self.screen.clone(), TextureOptions::NEAREST));
            }
            Some(tex) if dirty => tex.set(self.screen.clone(), TextureOptions::NEAREST),
            _ => {}
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context, state: &RunState) {
        // Debugger shortcuts work everywhere.
        let (toggle_sidebar, run_pause, step, step_over, reset) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::F1),
                i.consume_key(Modifiers::NONE, Key::F5),
                i.consume_key(Modifiers::NONE, Key::F11),
                i.consume_key(Modifiers::NONE, Key::F10),
                i.consume_key(Modifiers::NONE, Key::F6),
            )
        });
        if toggle_sidebar {
            self.set_sidebar(ctx, !self.sidebar_open);
        }
        if run_pause {
            self.link.send(if state.is_running() { Command::Pause } else { Command::Resume });
        }
        if step {
            self.link.send(Command::Step(1));
        }
        if step_over {
            self.link.send(Command::StepOver);
        }
        if reset {
            self.link.send(Command::Reset);
        }

        // Everything else goes to the emulated keyboard, unless a text field
        // in the sidebar has focus.
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let codes: Vec<u16> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    Event::Key { key, pressed: true, modifiers, .. }
                        if !modifiers.ctrl && !modifiers.alt && !modifiers.command =>
                    {
                        scan_code_of(*key)
                    }
                    _ => None,
                })
                .collect()
        });
        for code in codes {
            self.io.press_key(code);
        }
    }

    /// Open/close the sidebar, growing or shrinking the window so the screen
    /// keeps its size.
    fn set_sidebar(&mut self, ctx: &egui::Context, open: bool) {
        if open == self.sidebar_open {
            return;
        }
        self.sidebar_open = open;
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
            let delta = SIDEBAR_WIDTH - COLLAPSED_WIDTH;
            let w = rect.width() + if open { delta } else { -delta };
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(w, rect.height())));
        }
    }

    fn draw_screen(&mut self, ui: &mut egui::Ui) {
        let avail = ui.max_rect();
        ui.painter().rect_filled(avail, 0.0, Color32::BLACK);
        let Some(tex) = &self.texture else { return };

        // Largest integer scale that fits (crisp pixels), else shrink to fit.
        let fit = (avail.width() / FB_WIDTH as f32).min(avail.height() / FB_HEIGHT as f32);
        let scale = if fit >= 1.0 { fit.floor() } else { fit };
        let size = Vec2::new(FB_WIDTH as f32, FB_HEIGHT as f32) * scale;
        let rect = Rect::from_center_size(avail.center(), size);

        let response = ui.allocate_rect(rect, Sense::hover());
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        // Hover a cell (with the sidebar open) to see its VRAM address/value.
        if self.sidebar_open {
            if let Some(pos) = response.hover_pos() {
                let col = (((pos.x - rect.min.x) / scale) as usize / GLYPH_WIDTH).min(VGA_WIDTH - 1);
                let line = (((pos.y - rect.min.y) / scale) as usize / GLYPH_HEIGHT).min(VGA_HEIGHT - 1);
                let addr = line * VGA_WIDTH + col;
                let val = self.io.vram[addr].load(Ordering::Relaxed);
                let c = Cell::decode(val);
                response.on_hover_text_at_pointer(format!(
                    "col {col}, row {line}\nVRAM 0x{addr:04X} = 0x{val:04X}\nchar 0x{:02X} fg {} bg {}",
                    c.ch, c.fg, c.bg
                ));
            }
        }
    }

    fn update_stats(&mut self, ctx: &egui::Context, state: &RunState) {
        self.frames += 1;
        let elapsed = self.stats_since.elapsed();
        if elapsed >= Duration::from_millis(500) {
            let insts = self.io.running_count.swap(0, Ordering::Relaxed);
            self.rate = format_rate(insts, self.frames, elapsed.as_secs_f64());
            self.frames = 0;
            self.stats_since = Instant::now();
        }
        let status = match state {
            RunState::Running => self.rate.clone(),
            RunState::Paused(_) => "paused".to_string(),
            RunState::Halted => "halted".to_string(),
            RunState::Error(e) => format!("stopped: {e}"),
        };
        let title = format!("{TITLE} - {status}");
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }
}

impl eframe::App for WindowApp<'_> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.link.request_snapshot();
        let snap = self.link.snapshot();

        self.handle_input(&ctx, &snap.state);
        self.refresh_screen(&ctx);
        self.update_stats(&ctx, &snap.state);

        let mut open = self.sidebar_open;
        let mut want: Option<bool> = None;
        let rate = self.rate.clone();
        let panel = &mut self.panel;
        Panel::show_switched(
            ui,
            &mut open,
            Panel::right("debug_collapsed").exact_size(COLLAPSED_WIDTH).resizable(false),
            Panel::right("debug_expanded").default_size(SIDEBAR_WIDTH).size_range(320.0..=720.0).resizable(true),
            |ui, expanded| {
                if expanded {
                    if panel.show(ui, &snap, &rate) {
                        want = Some(false);
                    }
                } else if ui
                    .add(egui::Button::new("«").min_size(Vec2::new(COLLAPSED_WIDTH - 8.0, 40.0)))
                    .on_hover_text("Open debugger (F1)")
                    .clicked()
                {
                    want = Some(true);
                }
            },
        );
        // Dragging the panel edge can also open/close it.
        if open != self.sidebar_open {
            want = Some(open);
        }
        if let Some(w) = want {
            self.set_sidebar(&ctx, w);
        }

        egui::CentralPanel::no_frame().show(ui, |ui| self.draw_screen(ui));

        if self.opts.fps > 0 {
            ctx.request_repaint_after(Duration::from_secs_f64(1.0 / self.opts.fps as f64));
        } else {
            ctx.request_repaint();
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
