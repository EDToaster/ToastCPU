mod args;
mod devices;
mod emulator;
mod jit;
mod key;
mod vga;

use std::fs;

use args::get_args;
use emulator::ROM_SIZE;
use regex::Regex;

use crate::emulator::emulate;
use crate::jit::jit;

fn parse_program(prog: &str) -> Vec<u16> {
    let mut program: Vec<u16> = vec![0x7000; ROM_SIZE];

    /*
     *  DEPTH = 32768;                -- The size of memory in words
     *  WIDTH = 16;                   -- The size of data in bits
     *  ADDRESS_RADIX = HEX;          -- The radix for address values
     *  DATA_RADIX = HEX;             -- The radix for data values
     *  CONTENT                       -- start of (address : data pairs)
     *  BEGIN
     *  0000 : 2C08; -- imov r12 .isr
     */

    // assume we are using HEX with depth of 32k and width of 16

    let line_matcher = Regex::new(r"^([0-9a-fA-F]{4}) : ([0-9a-fA-F]{4});.*$").unwrap();

    for line in prog.lines() {
        let cap_opt = line_matcher.captures(line);
        if cap_opt.is_none() {
            continue;
        }

        let caps = cap_opt.unwrap();
        let addr = caps
            .get(1)
            .map(|m| u16::from_str_radix(m.as_str(), 16).unwrap())
            .unwrap();
        let val = caps
            .get(2)
            .map(|m| u16::from_str_radix(m.as_str(), 16).unwrap())
            .unwrap();

        program[usize::from(addr)] = val;
    }

    program
}

fn main() -> Result<(), String> {
    let opts = get_args().map_err(|e| format!("{e}"))?;

    let prog_string =
        fs::read_to_string(opts.mif_file).expect("Should have been able to read the file");

    let bytes = prog_string.len();
    println!("read {bytes} bytes from file");

    // INIT ROM
    print!("Parsing {bytes} bytes to instructions ... ");
    let rom: Vec<u16> = parse_program(&prog_string);
    println!("rom is sized {}", rom.len());

    if opts.jit_mode {
        jit(rom)?;
    } else {
        emulate(rom)?;
    }

    Ok(())
}
