mod cpu;
pub mod debugger;
pub mod disasm;

use std::io::stdout;
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::disable_raw_mode;

use crate::devices::Devices;
use crate::display::terminal::TerminalDisplay;
use crate::display::window::{WindowApp, WindowOptions};
use crate::hardware::def::*;
use crate::hardware::key::TerminalKeyboard;
use crate::io::SharedIo;
use crate::program::Program;
use debugger::{DebugLink, DebugRunner};

pub use cpu::Cpu;

/// Where the VGA text buffer is presented.
pub enum Frontend {
    /// ANSI escapes in the current terminal; keyboard input from the tty.
    Terminal,
    /// A native window with the FPGA font and a debugger sidebar; keyboard
    /// input from the window.
    Window(WindowOptions, DebugStart),
}

/// Initial debugger state for window mode.
#[derive(Default, Debug)]
pub struct DebugStart {
    pub paused: bool,
    /// Addresses or label names.
    pub breakpoints: Vec<String>,
}

pub fn emulate(program: Program, frontend: Frontend) -> Result<(), String> {
    let io = SharedIo::new();
    match frontend {
        Frontend::Terminal => run_terminal(program.rom, &io),
        Frontend::Window(opts, start) => run_window(program, &io, opts, start),
    }
}

fn run_cpu(rom: Vec<u16>, io: &SharedIo) -> Result<(), String> {
    let mut mem = Devices::new(rom, &io.vram, vec![0; RAM_SIZE], &io.key);
    Cpu::new().run(&mut mem, io)
}

/// Terminal mode: CPU on the main thread, display and keyboard on helpers.
fn run_terminal(rom: Vec<u16>, io: &SharedIo) -> Result<(), String> {
    let mut display = TerminalDisplay::new(VGA_WIDTH, VGA_HEIGHT, stdout(), io, Duration::from_millis(100));
    display.reset();
    let mut keyboard = TerminalKeyboard::new(io);

    let result = std::thread::scope(|scope| {
        scope.spawn(move || keyboard.handle());
        scope.spawn(move || display.start_loop());

        let result = run_cpu(rom, io);
        // Stop the helper threads whether we halted or hit an error; the scope
        // joins them before returning.
        io.terminate();
        result
    });

    execute!(stdout(), ResetColor, Show).unwrap();
    disable_raw_mode().unwrap();
    result
}

/// Window mode: the window owns the main thread (required for the event loop
/// on macOS), the CPU runs on a worker under the debugger's control. Closing
/// the window stops the CPU; if the program halts first, the window stays open
/// on the final screen so it can be inspected or reset.
fn run_window(program: Program, io: &SharedIo, opts: WindowOptions, start: DebugStart) -> Result<(), String> {
    let Program { rom, debug } = program;
    let (link, commands) = DebugLink::new();

    let mem = Devices::new(rom.clone(), &io.vram, vec![0; RAM_SIZE], &io.key);
    let mut runner = DebugRunner::new(mem, io, &link);
    if start.paused {
        runner.start_paused();
    }
    for bp in &start.breakpoints {
        let addr = debug.resolve(bp).ok_or_else(|| format!("unknown breakpoint address or label: {bp}"))?;
        runner.add_breakpoint(addr);
    }

    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("ToastCPU")
            .with_inner_size(WindowApp::initial_size(&opts))
            .with_min_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    std::thread::scope(|scope| {
        let cpu = scope.spawn(move || runner.run(commands));

        let ui_result = eframe::run_native(
            "ToastCPU",
            native,
            Box::new(|_cc| Ok(Box::new(WindowApp::new(io, &link, &rom, &debug, opts)))),
        );

        io.terminate();
        link.close();
        let cpu_result = cpu.join().map_err(|_| "CPU thread panicked".to_string())?;
        ui_result.map_err(|e| format!("could not open window: {e}"))?;
        cpu_result
    })
}
