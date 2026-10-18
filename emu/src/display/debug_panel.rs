//! The debugger sidebar shown in window mode.

use std::sync::atomic::Ordering;

use eframe::egui::{
    self, Color32, CollapsingHeader, ComboBox, DragValue, Grid, RichText, ScrollArea, TextEdit,
    TextStyle, Ui,
};

use crate::emulator::debugger::{Command, DebugLink, PauseReason, RunState, Snapshot};
use crate::emulator::disasm::{disassemble, REG_NAMES};
use crate::hardware::def::{RAM_SIZE, ROM_SIZE, VRAM_SIZE};
use crate::hardware::register::{StatusRegisterFlag, REG_PC, REG_SP, REG_SR};
use crate::io::SharedIo;
use crate::program::DebugInfo;

const SPEEDS: [(Option<u64>, &str); 8] = [
    (None, "Full speed"),
    (Some(10_000_000), "10M ips"),
    (Some(1_000_000), "1M ips"),
    (Some(100_000), "100k ips"),
    (Some(10_000), "10k ips"),
    (Some(1_000), "1k ips"),
    (Some(100), "100 ips"),
    (Some(10), "10 ips"),
];

const CHANGED: Color32 = Color32::from_rgb(255, 200, 60);
const PC_ROW: Color32 = Color32::from_rgb(60, 70, 30);
const BREAK: Color32 = Color32::from_rgb(230, 70, 70);
const LABEL: Color32 = Color32::from_rgb(120, 180, 255);
const DIM: Color32 = Color32::from_gray(130);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Ram,
    Rom,
    Vram,
}

impl Region {
    fn base(self) -> u16 {
        match self {
            Region::Ram => 0x8000,
            Region::Rom | Region::Vram => 0x0000,
        }
    }
    fn len(self) -> usize {
        match self {
            Region::Ram => RAM_SIZE,
            Region::Rom => ROM_SIZE,
            Region::Vram => VRAM_SIZE,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Region::Ram => "RAM 8000-BFFF",
            Region::Rom => "ROM 0000-7FFF",
            Region::Vram => "VRAM (write-only)",
        }
    }
}

const WORDS_PER_ROW: usize = 8;

pub struct DebugPanel<'a> {
    io: &'a SharedIo,
    link: &'a DebugLink,
    rom: &'a [u16],
    debug: &'a DebugInfo,

    // registers as of the previous stop, to highlight what changed
    stop_regs: [u16; 16],
    changed: [bool; 16],
    seen_retired: u64,

    follow_pc: bool,
    disasm_scroll_to: Option<u16>,
    last_pc: u16,
    disasm_goto: String,

    address_input: String,
    address_error: bool,
    step_n: u32,

    region: Region,
    mem_goto: String,
    mem_scroll_to: Option<usize>,
}

impl<'a> DebugPanel<'a> {
    pub fn new(io: &'a SharedIo, link: &'a DebugLink, rom: &'a [u16], debug: &'a DebugInfo) -> Self {
        DebugPanel {
            io,
            link,
            rom,
            debug,
            stop_regs: [0; 16],
            changed: [false; 16],
            seen_retired: u64::MAX,
            follow_pc: true,
            disasm_scroll_to: Some(0),
            last_pc: u16::MAX,
            disasm_goto: String::new(),
            address_input: String::new(),
            address_error: false,
            step_n: 100,
            region: Region::Ram,
            mem_goto: String::new(),
            mem_scroll_to: None,
        }
    }

    fn track_changes(&mut self, snap: &Snapshot) {
        if snap.state.is_running() {
            self.changed = [false; 16];
            return;
        }
        if snap.retired != self.seen_retired {
            for i in 0..16 {
                self.changed[i] = self.seen_retired != u64::MAX && snap.regs[i] != self.stop_regs[i];
            }
            self.stop_regs = snap.regs;
            self.seen_retired = snap.retired;
        }
    }

    /// Draw the sidebar. Returns true if the user asked to collapse it.
    pub fn show(&mut self, ui: &mut Ui, snap: &Snapshot, rate: &str) -> bool {
        self.track_changes(snap);
        let pc = snap.regs[REG_PC as usize];
        if self.follow_pc && pc != self.last_pc {
            self.disasm_scroll_to = Some(pc);
        }
        self.last_pc = pc;

        // Children (truncating labels, grids) must not widen the panel.
        ui.set_max_width(ui.available_width());

        let mut close = false;
        ui.horizontal(|ui| {
            if ui.button("»").on_hover_text("Collapse (F1)").clicked() {
                close = true;
            }
            ui.heading("Debugger");
        });
        ui.separator();

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            self.status(ui, snap, rate);
            ui.add_space(4.0);
            self.controls(ui, snap);
            ui.separator();
            CollapsingHeader::new("Registers").default_open(true).show(ui, |ui| self.registers(ui, snap));
            CollapsingHeader::new("Disassembly").default_open(true).show(ui, |ui| self.disassembly(ui, snap));
            CollapsingHeader::new(format!("Breakpoints ({})", snap.breakpoints.len()))
                .default_open(true)
                .show(ui, |ui| self.breakpoints(ui, snap));
            CollapsingHeader::new("Stack").default_open(true).show(ui, |ui| self.stack(ui, snap));
            CollapsingHeader::new("Memory").default_open(false).show(ui, |ui| self.memory(ui, snap));
        });
        close
    }

    fn status(&self, ui: &mut Ui, snap: &Snapshot, rate: &str) {
        let (text, color) = match &snap.state {
            RunState::Running => ("RUNNING".to_string(), Color32::from_rgb(80, 200, 120)),
            RunState::Paused(reason) => {
                let why = match reason {
                    PauseReason::Startup => "at start".to_string(),
                    PauseReason::User => "by user".to_string(),
                    PauseReason::Step => "after step".to_string(),
                    PauseReason::Reset => "after reset".to_string(),
                    PauseReason::Breakpoint(a) => format!("breakpoint {}", self.describe(*a)),
                };
                (format!("PAUSED ({why})"), CHANGED)
            }
            RunState::Halted => ("HALTED".to_string(), Color32::from_rgb(150, 150, 255)),
            RunState::Error(_) => ("ERROR".to_string(), BREAK),
        };
        ui.label(RichText::new(text).strong().color(color));
        if let RunState::Error(e) = &snap.state {
            ui.label(RichText::new(e).color(BREAK).monospace());
        }
        ui.horizontal(|ui| {
            if snap.state.is_running() {
                ui.monospace(rate);
                ui.separator();
            }
            ui.monospace(format!("{} instructions", group_digits(snap.retired)));
        });
        ui.horizontal(|ui| {
            ui.monospace(format!("key 0x{:04X}", snap.key));
            if snap.irq_pending {
                ui.label(RichText::new("IRQ pending").color(CHANGED));
            }
        });
    }

    fn controls(&mut self, ui: &mut Ui, snap: &Snapshot) {
        let stopped = snap.state.can_step();
        ui.horizontal_wrapped(|ui| {
            if snap.state.is_running() {
                if ui.button("Pause").on_hover_text("F5").clicked() {
                    self.link.send(Command::Pause);
                }
            } else if ui.add_enabled(stopped, egui::Button::new("Continue")).on_hover_text("F5").clicked() {
                self.link.send(Command::Resume);
            }
            if ui.add_enabled(stopped, egui::Button::new("Step")).on_hover_text("Step one instruction (F11)").clicked() {
                self.link.send(Command::Step(1));
            }
            if ui
                .add_enabled(stopped, egui::Button::new("Step over"))
                .on_hover_text("Step, running jmpl calls to completion (F10)")
                .clicked()
            {
                self.link.send(Command::StepOver);
            }
            if ui.button("Reset").on_hover_text("Reset CPU, RAM and screen; keeps breakpoints (F6)").clicked() {
                self.link.send(Command::Reset);
            }
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(stopped, egui::Button::new("Step ×")).clicked() {
                self.link.send(Command::Step(self.step_n));
            }
            ui.add(DragValue::new(&mut self.step_n).range(1..=10_000_000).speed(10));
            ui.separator();
            ui.label("Speed");
            let current = SPEEDS.iter().find(|(s, _)| *s == snap.speed).map_or("custom", |(_, n)| n);
            ComboBox::from_id_salt("speed").selected_text(current).width(100.0).show_ui(ui, |ui| {
                for (speed, name) in SPEEDS {
                    if ui.selectable_label(snap.speed == speed, name).clicked() {
                        self.link.send(Command::SetSpeed(speed));
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            let edit = TextEdit::singleline(&mut self.address_input)
                .hint_text("address or label")
                .desired_width(150.0)
                .font(TextStyle::Monospace);
            let edit = if self.address_error { edit.text_color(BREAK) } else { edit };
            let resp = ui.add(edit);
            if resp.changed() {
                self.address_error = false;
            }
            let addr = self.debug.resolve(&self.address_input);
            let run_to = ui.add_enabled(stopped, egui::Button::new("Run to")).clicked();
            let add_bp = ui.button("Toggle breakpoint").clicked()
                || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if run_to || add_bp {
                match addr {
                    Some(a) if run_to => self.link.send(Command::RunTo(a)),
                    Some(a) => self.link.send(Command::ToggleBreakpoint(a)),
                    None => self.address_error = true,
                }
            }
        });
    }

    fn registers(&mut self, ui: &mut Ui, snap: &Snapshot) {
        let editable = !snap.state.is_running();
        Grid::new("regs").num_columns(4).striped(true).spacing([10.0, 2.0]).show(ui, |ui| {
            for (i, name) in REG_NAMES.iter().enumerate() {
                let color = if self.changed[i] { CHANGED } else { ui.visuals().text_color() };
                ui.label(RichText::new(format!("r{i:<2} {name}")).monospace().color(color));

                let mut v = snap.regs[i];
                let resp = ui.add_enabled(
                    editable,
                    DragValue::new(&mut v).hexadecimal(4, false, true).prefix("0x").speed(0.0),
                );
                if resp.changed() && v != snap.regs[i] {
                    self.link.send(Command::SetRegister(i as u16, v));
                }
                ui.label(RichText::new(format!("{:>6}", snap.regs[i] as i16)).monospace().color(DIM));
                let note = if i == REG_SR as usize {
                    String::new()
                } else if snap.regs[i] != 0
                    && (snap.regs[i] as usize) < ROM_SIZE
                    && self.debug.labels.contains_key(&snap.regs[i])
                {
                    self.debug.labels[&snap.regs[i]].join(", ")
                } else {
                    String::new()
                };
                ui.add(egui::Label::new(RichText::new(note).monospace().color(LABEL).small()).truncate());
                ui.end_row();
            }
        });
        let sr = snap.regs[REG_SR as usize];
        ui.horizontal(|ui| {
            ui.label("flags");
            for (flag, name) in [
                (StatusRegisterFlag::V, "V"),
                (StatusRegisterFlag::C, "C"),
                (StatusRegisterFlag::N, "N"),
                (StatusRegisterFlag::Z, "Z"),
                (StatusRegisterFlag::X, "X"),
            ] {
                let on = flag.read(sr);
                let text = RichText::new(name).monospace().strong();
                ui.label(if on { text.color(Color32::BLACK).background_color(CHANGED) } else { text.color(DIM) });
            }
        });
    }

    /// "fn_main+3 (0x06A8)" or just "0x06A8".
    fn describe(&self, addr: u16) -> String {
        match self.debug.symbolize(addr) {
            Some(sym) => format!("{sym} (0x{addr:04X})"),
            None => format!("0x{addr:04X}"),
        }
    }

    fn disassembly(&mut self, ui: &mut Ui, snap: &Snapshot) {
        let pc = snap.regs[REG_PC as usize];
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.follow_pc, "Follow PC");
            let resp = ui.add(
                TextEdit::singleline(&mut self.disasm_goto)
                    .hint_text("go to address/label")
                    .desired_width(150.0)
                    .font(TextStyle::Monospace),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(a) = self.debug.resolve(&self.disasm_goto) {
                    self.follow_pc = false;
                    self.disasm_scroll_to = Some(a);
                }
            }
        });
        ui.label(RichText::new("click the gutter to toggle a breakpoint; right-click a row for more").small().color(DIM));

        let row_h = ui.text_style_height(&TextStyle::Monospace);
        let spacing = ui.spacing().item_spacing.y;
        let mut area = ScrollArea::vertical().id_salt("disasm").max_height(row_h * 18.0).auto_shrink([false, true]);
        if let Some(target) = self.disasm_scroll_to.take() {
            let row = target.min(ROM_SIZE as u16 - 1) as f32;
            area = area.vertical_scroll_offset(((row - 6.0).max(0.0)) * (row_h + spacing));
        }
        area.show_rows(ui, row_h, ROM_SIZE, |ui, rows| {
            for addr in rows {
                let addr = addr as u16;
                let word = self.rom[addr as usize];
                let is_bp = snap.breakpoints.contains(&addr);
                let is_pc = addr == pc;

                // Reserve a background shape so the PC highlight sits behind the text.
                let bg = ui.painter().add(egui::Shape::Noop);
                let row = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let gutter = ui.add(
                        egui::Label::new(
                            RichText::new(if is_bp { "●" } else { "·" })
                                .monospace()
                                .color(if is_bp { BREAK } else { DIM }),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if gutter.clicked() {
                        self.link.send(Command::ToggleBreakpoint(addr));
                    }
                    let marker = if is_pc { "▶" } else { " " };
                    ui.monospace(RichText::new(format!("{marker}{addr:04X}")).color(if is_pc { CHANGED } else { DIM }));
                    ui.monospace(RichText::new(format!("{word:04X}")).color(DIM));
                    ui.monospace(format!("{:<17}", disassemble(word)));
                    // Labels then source; truncated to the panel width.
                    let mut tail = String::new();
                    if let Some(labels) = self.debug.labels.get(&addr) {
                        tail.push_str(&format!("{}: ", labels.join(", ")));
                    }
                    let label_len = tail.len();
                    if let Some(src) = self.debug.source.get(&addr) {
                        tail.push_str(src);
                    }
                    if !tail.is_empty() {
                        let color = if label_len > 0 { LABEL } else { DIM };
                        let full = tail.clone();
                        ui.add(egui::Label::new(RichText::new(tail).monospace().color(color)).truncate())
                            .on_hover_text(full);
                    }
                });
                if is_pc {
                    let rect = row.response.rect.expand(1.0);
                    ui.painter().set(bg, egui::Shape::rect_filled(rect, 2.0, PC_ROW));
                }
                row.response.context_menu(|ui| {
                    if ui.button("Toggle breakpoint").clicked() {
                        self.link.send(Command::ToggleBreakpoint(addr));
                        ui.close();
                    }
                    if ui.add_enabled(snap.state.can_step(), egui::Button::new("Run to here")).clicked() {
                        self.link.send(Command::RunTo(addr));
                        ui.close();
                    }
                    if ui.add_enabled(!snap.state.is_running(), egui::Button::new("Set PC here")).clicked() {
                        self.link.send(Command::SetRegister(REG_PC, addr));
                        ui.close();
                    }
                });
            }
        });
    }

    fn breakpoints(&mut self, ui: &mut Ui, snap: &Snapshot) {
        if snap.breakpoints.is_empty() {
            ui.label(RichText::new("none - click the disassembly gutter or use the field above").color(DIM));
            return;
        }
        for &addr in &snap.breakpoints {
            ui.horizontal(|ui| {
                if ui.small_button("x").on_hover_text("Remove").clicked() {
                    self.link.send(Command::ToggleBreakpoint(addr));
                }
                if ui.link(RichText::new(self.describe(addr)).monospace()).clicked() {
                    self.follow_pc = false;
                    self.disasm_scroll_to = Some(addr);
                }
            });
        }
        if ui.button("Clear all").clicked() {
            self.link.send(Command::ClearBreakpoints);
        }
    }

    fn stack(&self, ui: &mut Ui, snap: &Snapshot) {
        let sp = snap.regs[REG_SP as usize];
        if !(0x8000..=0xBFFF).contains(&sp) {
            ui.label(RichText::new(format!("sp = 0x{sp:04X} is outside RAM")).color(BREAK));
            return;
        }
        let depth = 0xBFFF - sp;
        ui.label(RichText::new(format!("sp = 0x{sp:04X}, {depth} words deep")).monospace().color(DIM));
        Grid::new("stack").num_columns(3).striped(true).show(ui, |ui| {
            for addr in sp..sp.saturating_add(12).min(0xBFFF) {
                let val = snap.ram[(addr - 0x8000) as usize];
                ui.monospace(RichText::new(format!("{addr:04X}")).color(DIM));
                ui.monospace(format!("{val:04X}"));
                let sym = if val != 0 && (val as usize) < ROM_SIZE { self.debug.symbolize_within(val, 32) } else { None };
                ui.add(egui::Label::new(RichText::new(sym.unwrap_or_default()).monospace().color(LABEL).small()).truncate());
                ui.end_row();
            }
        });
    }

    fn memory(&mut self, ui: &mut Ui, snap: &Snapshot) {
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("region").selected_text(self.region.name()).show_ui(ui, |ui| {
                for r in [Region::Ram, Region::Rom, Region::Vram] {
                    if ui.selectable_label(self.region == r, r.name()).clicked() {
                        self.region = r;
                        self.mem_scroll_to = Some(0);
                    }
                }
            });
            let resp = ui.add(
                TextEdit::singleline(&mut self.mem_goto)
                    .hint_text("go to")
                    .desired_width(90.0)
                    .font(TextStyle::Monospace),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(a) = self.debug.resolve(&self.mem_goto) {
                    self.region = match a {
                        0x8000..=0xBFFF => Region::Ram,
                        _ => Region::Rom,
                    };
                    self.mem_scroll_to = Some(a.wrapping_sub(self.region.base()) as usize);
                }
            }
        });

        let sp = snap.regs[REG_SP as usize];
        let region = self.region;
        let read = |i: usize| -> u16 {
            match region {
                Region::Ram => snap.ram[i],
                Region::Rom => self.rom[i],
                Region::Vram => self.io.vram[i].load(Ordering::Relaxed),
            }
        };

        let row_h = ui.text_style_height(&TextStyle::Monospace);
        let spacing = ui.spacing().item_spacing.y;
        let rows = region.len().div_ceil(WORDS_PER_ROW);
        let mut area = ScrollArea::vertical().id_salt("memory").max_height(row_h * 16.0).auto_shrink([false, true]);
        if let Some(i) = self.mem_scroll_to.take() {
            area = area.vertical_scroll_offset((i / WORDS_PER_ROW) as f32 * (row_h + spacing));
        }
        area.show_rows(ui, row_h, rows, |ui, range| {
            for row in range {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    let start = row * WORDS_PER_ROW;
                    ui.monospace(RichText::new(format!("{:04X}", region.base() as usize + start)).color(DIM));
                    let mut ascii = String::new();
                    for i in start..(start + WORDS_PER_ROW).min(region.len()) {
                        let val = read(i);
                        let addr = (region.base() as usize + i) as u16;
                        let text = RichText::new(format!("{val:04X}")).monospace();
                        let text = if region == Region::Ram && addr == sp {
                            text.color(Color32::BLACK).background_color(CHANGED)
                        } else if val == 0 {
                            text.color(DIM)
                        } else {
                            text
                        };
                        ui.label(text);
                        let ch = (val & 0xFF) as u8;
                        ascii.push(if (0x20..0x7F).contains(&ch) { ch as char } else { '.' });
                    }
                    ui.monospace(RichText::new(ascii).color(DIM));
                });
            }
        });
    }
}

fn group_digits(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
