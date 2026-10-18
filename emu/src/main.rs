mod args;
mod devices;
mod display;
mod emulator;
mod hardware;
mod io;
mod jit;
mod program;

use std::fs;

use args::get_args;

use crate::display::window::WindowOptions;
use crate::emulator::{emulate, DebugStart, Frontend};
use crate::jit::jit;
use crate::program::Program;

fn main() -> Result<(), String> {
    let opts = get_args().map_err(|e| format!("{e}"))?;

    let prog_string = fs::read_to_string(&opts.mif_file)
        .map_err(|e| format!("could not read {}: {e}", opts.mif_file))?;

    let bytes = prog_string.len();
    println!("read {bytes} bytes from file");

    // INIT ROM
    print!("Parsing {bytes} bytes to instructions ... ");
    let program = Program::parse(&prog_string);
    println!("rom is sized {}", program.rom.len());

    if opts.jit_mode {
        return jit(program.rom);
    }

    let window_only = opts.debug || opts.paused || !opts.breakpoints.is_empty();
    let frontend = if opts.window || window_only {
        Frontend::Window(
            WindowOptions { scale: opts.scale.max(1), fps: opts.fps, debug: window_only },
            DebugStart {
                paused: opts.paused,
                breakpoints: opts
                    .breakpoints
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect(),
            },
        )
    } else {
        Frontend::Terminal
    };
    emulate(program, frontend)
}
