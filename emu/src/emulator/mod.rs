mod cpu;

use std::io::stdout;
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::disable_raw_mode;

use crate::devices::Devices;
use crate::display::terminal::TerminalDisplay;
use crate::display::window::{WindowDisplay, WindowOptions};
use crate::hardware::def::*;
use crate::hardware::key::TerminalKeyboard;
use crate::io::SharedIo;

pub use cpu::Cpu;

/// Where the VGA text buffer is presented.
pub enum Frontend {
    /// ANSI escapes in the current terminal; keyboard input from the tty.
    Terminal,
    /// A native window with the FPGA font; keyboard input from the window.
    Window(WindowOptions),
}

pub fn emulate(rom: Vec<u16>, frontend: Frontend) -> Result<(), String> {
    let io = SharedIo::new();
    match frontend {
        Frontend::Terminal => run_terminal(rom, &io),
        Frontend::Window(opts) => run_window(rom, &io, opts),
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
/// on macOS), the CPU runs on a worker. Closing the window stops the CPU; if
/// the program halts first, the window stays open showing the final screen.
fn run_window(rom: Vec<u16>, io: &SharedIo, opts: WindowOptions) -> Result<(), String> {
    let mut display = WindowDisplay::new(io, opts)?;

    std::thread::scope(|scope| {
        let cpu = scope.spawn(move || run_cpu(rom, io));

        display.run();
        io.terminate();

        cpu.join().map_err(|_| "CPU thread panicked".to_string())?
    })
}
