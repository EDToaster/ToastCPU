mod devices;
mod diagnostics;
mod key;
mod vga;

use std::env;
use std::fs;
use std::io::stdout;
use std::ops::{Index, IndexMut};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use regex::Regex;
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use vga::Vga;

use crate::devices::Devices;
use crate::diagnostics::Diagnostics;
use crate::key::Key;

const ROM_SIZE: usize = 0x8000;
const RAM_SIZE: usize = 0x4000;
const VGA_WIDTH: usize = 100;
const VGA_HEIGHT: usize = 60;

const LOAD: u16 = 0;
const STR: u16 = 1;
const IMOV: u16 = 2;
const IMOH: u16 = 3;
const PUSH: u16 = 5;
const POP: u16 = 6;
const HALT: u16 = 7;
const ALU: u16 = 8;
const IALU: u16 = 9;
const JMP: u16 = 10;
const RTI: u16 = 12;

enum StatusRegisterFlag {
    X,
    Z,
    N,
    C,
    V,
}
impl StatusRegisterFlag {
    fn mask(&self) -> u16 {
        match *self {
            StatusRegisterFlag::X => 0x0001,
            StatusRegisterFlag::Z => 0x0002,
            StatusRegisterFlag::N => 0x0004,
            StatusRegisterFlag::C => 0x0008,
            StatusRegisterFlag::V => 0x0010,
        }
    }
}

/**
   if (set_VC) SR[4:3] <= {V, C};
   SR[2:0] <= {N, Z, X};
*/
struct StatusRegister {
    sr: u16,
}

impl StatusRegister {
    fn get(&self, flag: StatusRegisterFlag) -> bool {
        (self.sr & flag.mask()) != 0
    }

    fn set(&mut self, flag: StatusRegisterFlag, val: bool) {
        if val {
            self.sr |= flag.mask();
        } else {
            self.sr &= !flag.mask();
        }
    }
}
struct Registers {
    general: [u16; 12],
    // # r12 - ISR
    // # r13 - SP
    // # r14 - SR
    // # r15 - PC
    isr: u16,
    sp: u16,
    sr: StatusRegister,
    pc: u16,
}

impl Index<u16> for Registers {
    type Output = u16;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0..=11 => &self.general[index as usize],
            12 => &self.isr,
            13 => &self.sp,
            14 => &self.sr.sr,
            15 => &self.pc,
            _ => panic!("Register index {index} is out of bounds!"),
        }
    }
}

impl IndexMut<u16> for Registers {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0..=11 => &mut self.general[index as usize],
            12 => &mut self.isr,
            13 => &mut self.sp,
            14 => &mut self.sr.sr,
            15 => &mut self.pc,
            _ => panic!("Register index {index} is out of bounds!"),
        }
    }
}

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

fn bit(n: i32, bit: u8) -> bool {
    (n >> bit) & 1 != 0
}

fn alu(op: u16, a: i32, b: i32, sr: &mut StatusRegister) -> u16 {
    let agg: i32 = match op {
        0x0 => !a,
        0x1 => a & b,
        0x2 => a | b,
        0x3 => a ^ b,
        0x4 => a + b,
        0x5 => a - b,
        0x6 => b,
        0x7 => a - b,
        0x8 => ((a as u16) >> b) as i32,
        0x9 => ((a as i16) >> b) as i32,
        0xA => a << b,
        _ => panic!("Invalid alu operation {op}"),
    };

    sr.set(StatusRegisterFlag::N, agg & 0x8000 != 0);
    sr.set(StatusRegisterFlag::Z, agg == 0);
    sr.set(StatusRegisterFlag::X, agg == 0xFFFF);

    match op {
        0x4 => {
            sr.set(
                StatusRegisterFlag::V,
                (bit(a, 15) == bit(b, 15)) && (bit(a, 15) ^ bit(agg, 15)),
            );
            sr.set(StatusRegisterFlag::C, bit(agg, 16));
        }
        0x5 | 0x7 => {
            sr.set(
                StatusRegisterFlag::V,
                (bit(a, 15) ^ bit(b, 15)) && (bit(a, 15) != bit(agg, 15)),
            );
            sr.set(StatusRegisterFlag::C, bit(agg, 16));
        }
        _ => (),
    }

    (agg & 0xFFFF) as u16
}

fn hex_vec(v: &Vec<u16>) -> String {
    let mut s: String = String::new();
    for i in v {
        s.push_str(format!("{i:#06x}, ").as_str());
    }
    s
}

fn main() -> Result<(), String> {
    let mut buffer: AllocRingBuffer<u16> = AllocRingBuffer::with_capacity(128);
    emulate(&mut buffer).map_err(|e| format!("{e}: {:?}", hex_vec(&buffer.to_vec())))
}

#[allow(unused_variables)]
fn emulate(pc_buffer: &mut AllocRingBuffer<u16>) -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        let progname = &args[0];
        println!("Usage: {progname} MIF_FILE");
        process::exit(1);
    }

    let mif_file = &args[1];

    print!("Reading file {mif_file} from memory as rom ... ");
    let prog_string = fs::read_to_string(mif_file).expect("Should have been able to read the file");

    let bytes = prog_string.len();
    println!("read {bytes} bytes from file");

    // INIT ROM
    print!("Parsing {bytes} bytes to instructions ... ");
    let rom: Vec<u16> = parse_program(&prog_string);
    println!("rom is sized {}", rom.len());

    let ram: Vec<u16> = vec![0; RAM_SIZE];

    let mut diag_vga: Vga = Vga::new(VGA_WIDTH, VGA_HEIGHT, stdout());
    let mut mem_vga: Vga = Vga::new(VGA_WIDTH, VGA_HEIGHT, stdout());
    diag_vga.reset();
    mem_vga.reset();

    let key: Arc<Mutex<u16>> = Arc::new(Mutex::new(0));
    let irq: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // INIT REGISTERS
    let mut registers: Registers = Registers {
        general: [0; 12],
        isr: 0,
        sp: 0xBFFF,
        sr: StatusRegister { sr: 0 },
        pc: 0x0000,
    };

    let mut halt: bool = false;

    // start keyboard thread
    let mut key_handler = Key::new(Arc::clone(&irq), Arc::clone(&key));
    let key_handler_thread = thread::spawn(move || key_handler.handle());

    let mut diagnostics: Diagnostics =
        Diagnostics::new(diag_vga, Duration::new(0, 100_000_000));

    // main thread
    let mut mem = Devices::new(rom, mem_vga, ram, key);
    while !halt {
        if *irq.lock().unwrap() {
            *irq.lock().unwrap() = false;
            registers.sp -= 1;
            mem.write(registers.sp, registers.pc).map_err(|e| {
                format!(
                    "Issue when jumping to isr storing stack pointer instruction pc={:#06x}: {e}",
                    registers.pc
                )
            })?;
            registers.sp -= 1;
            mem.write(registers.sp, registers.sr.sr).map_err(|e| {
                format!(
                    "Issue when jumping to isr storing status register instruction pc={:#06x}: {e}",
                    registers.pc
                )
            })?;
            registers.pc = registers.isr;
            continue;
        }

        pc_buffer.push(registers.pc);

        let inst: u16 = mem
            .read(registers.pc)
            .map_err(|e| format!("Issue when read instruction pc={:#06x}: {e}", registers.pc))?;
        let opcode: u16 = (inst & 0xF000) >> 12;

        let r1: u16 = (inst & 0x0F00) >> 8;
        let r2: u16 = (inst & 0x00F0) >> 4;
        let imoh_imm8: u16 = (inst & 0x00FF) << 8;
        let imov_imm8: u16 = (inst & 0x00FF) | (if (inst & 0x0080) == 0 { 0x0000 } else { 0xFF00 });

        let alu_imm4: u16 = r2;
        let alu_op: u16 = inst & 0x000F;
        let load_offset: u16 = alu_op;
        let jmp_op: u16 = alu_op;

        match opcode {
            LOAD => {
                registers[r1] = mem.read(registers[r2] + load_offset).map_err(|e| {
                    format!("Issue when executing load at pc={:#06x}: {e}", registers.pc)
                })?;
            }
            STR => {
                mem.write(registers[r1] + load_offset, registers[r2])
                    .map_err(|e| {
                        format!(
                            "Issue when executing store at pc={:#06x}: {e}",
                            registers.pc
                        )
                    })?;
            }
            IMOV => {
                registers[r1] = imov_imm8;
            }
            IMOH => {
                registers[r1] = imoh_imm8 | (registers[r1] & 0x00FF);
            }
            PUSH => {
                registers[r1] -= 1;
                mem.write(registers[r1], registers[r2]).map_err(|e| {
                    format!("Issue when executing push at pc={:#06x}: {e}", registers.pc)
                })?;
            }
            POP => {
                registers[r1] = mem.read(registers[r2]).map_err(|e| {
                    format!("Issue when executing pop at pc={:#06x}: {e}", registers.pc)
                })?;
                registers[r2] += 1;
            }
            HALT => {
                halt = true;
            }
            ALU => {
                let agg = alu(
                    alu_op,
                    registers[r1] as i32,
                    registers[r2] as i32,
                    &mut registers.sr,
                );

                if alu_op != 0x7 {
                    registers[r1] = agg;
                }
            }
            IALU => {
                let agg = alu(
                    alu_op,
                    registers[r1] as i32,
                    alu_imm4 as i32,
                    &mut registers.sr,
                );

                if alu_op != 0x7 {
                    registers[r1] = agg;
                }
            }
            JMP => {
                let l: bool = r2 & 1 != 0;
                let r: bool = r2 & 2 != 0;

                let do_jump: bool = match jmp_op {
                    0 => true,
                    1 => registers.sr.get(StatusRegisterFlag::Z),
                    2 => !registers.sr.get(StatusRegisterFlag::Z),
                    3 => registers.sr.get(StatusRegisterFlag::N),
                    4 => {
                        !registers.sr.get(StatusRegisterFlag::Z)
                            && !registers.sr.get(StatusRegisterFlag::N)
                    }
                    _ => false,
                };

                if do_jump {
                    if r {
                        registers.pc = mem.read(registers.sp).map_err(|e| {
                            format!("Issue when executing jump at pc={:#06x}: {e}", registers.pc)
                        })?;
                        registers.sp += 1;
                    } else if l {
                        registers.sp -= 1;
                        mem.write(registers.sp, registers.pc + 1).map_err(|e| {
                            format!(
                                "Issue when executing jump and link at pc={:#06x}: {e}",
                                registers.pc
                            )
                        })?;

                        registers.pc = registers[r1];
                    } else {
                        registers.pc = registers[r1];
                    }
                    registers.pc -= 1;
                }
            }
            RTI => {
                registers.sr.sr = mem.read(registers.sp)
                    .map_err(|e| format!("Issue when executing rti and popping the status register at pc={:#06x}: {e}", registers.pc))?;
                registers.sp += 1;
                registers.pc = mem.read(registers.sp)
                    .map_err(|e| format!("Issue when executing rti and popping the return address at pc={:#06x}: {e}", registers.pc))?;
                registers.sp += 1;

                registers.pc -= 1;
            }
            _ => (),
        }
        registers.pc += 1;
        diagnostics.increment();
    }

    let last_pc = registers.pc - 1;
    diagnostics.halt(last_pc);

    // let v0 = registers[5];

    // println!("Program halted at PC={last_pc:04x}");
    // println!("Program halted at v0={v0:04x}");

    key_handler_thread.join().unwrap();

    Ok(())
}
