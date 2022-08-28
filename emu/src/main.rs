use std::env;
use std::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;
use std::process;
use std::fs;

use regex::Regex;

const ROM_SIZE: usize = 0x8000;
const RAM_SIZE: usize = 0x4000;
const VGA_WIDTH: usize = 100;
const VGA_HEIGHT: usize = 60;
const VGA_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;

const LOAD: u16 = 0;
const STR: u16 = 1;
const IMOV: u16 = 2;
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
    V   
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
            _ => panic!("Register index {index} is out of bounds!")
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
            _ => panic!("Register index {index} is out of bounds!")
        }
    }
}

struct Memory<'t> {
    rom: &'t mut Vec<u16>,
    vga: &'t mut Vec<u16>,
    ram: &'t mut Vec<u16>,
}

impl Memory<'_> {
    fn mem_read(&self, addr: u16) -> u16 {
        match addr {
            0..=0x7FFF => self.rom[addr as usize] as u16,
            0x8000..=0xBFFF => self.ram[(addr - 0x8000) as usize] as u16,
            _ => todo!("Memory location {addr:#06x}"),
        }
    }

    fn mem_write(&mut self, addr: u16, val: u16) {
        match addr {
            0..=0x7FFF => self.vga[addr as usize] = val,
            0x8000..=0xBFFF => self.ram[(addr - 0x8000) as usize] = val,
            _ => todo!("Memory location {addr:#06x}={val:#06x}"),
        }
    }
}

fn parse_program(prog: &String) -> Vec<u16> {
    let mut program: Vec<u16> = vec![0; ROM_SIZE];

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
        if let None = cap_opt {
            continue;
        }

        let caps = cap_opt.unwrap();
        let addr = caps.get(1).map(|m| u16::from_str_radix(m.as_str(), 16).unwrap()).unwrap();
        let val = caps.get(2).map(|m| u16::from_str_radix(m.as_str(), 16).unwrap()).unwrap();

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
            0x8 => ((a as u16) >> b) as i32,
            0x9 => a >> b,
            0xA => a << b,
            _ => panic!("Invalid alu operation {op}")
        };


        sr.set(StatusRegisterFlag::N, agg & 0x8000 != 0);
        sr.set(StatusRegisterFlag::Z, agg == 0);
        sr.set(StatusRegisterFlag::X, agg == 0xFFFF);

        match op {
            0x4 => {
                sr.set(StatusRegisterFlag::V, (bit(a, 15) == bit(b, 15)) && 
                    (bit(a, 15) ^ bit(agg, 15)));
                sr.set(StatusRegisterFlag::C, bit(agg, 16));
            }
            0x5 => {
                sr.set(StatusRegisterFlag::V, (bit(a, 15) ^ bit(b, 15)) && 
                (bit(a, 15) != bit(agg, 15)));
                sr.set(StatusRegisterFlag::C, bit(agg, 16));
            },
            _ => ()
        }

        let ret = (agg & 0xFFFF) as u16;
        println!("{a:#010x} {op} {b:#010x} = {ret:#06x}");
        ret
}

#[allow(unused_variables)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        let progname = &args[0];
        println!("Usage: {progname} MIF_FILE");
        process::exit(1);
    }

    let mif_file = &args[1];

    print!("Reading file {mif_file} from memory as rom ... ");
    let prog_string = fs::read_to_string(mif_file)
        .expect("Should have been able to read the file");

    let bytes = prog_string.len();
    println!("read {} bytes from file", bytes);

    // INIT ROM
    print!("Parsing {} bytes to instructions ... ", bytes);
    let mut rom: Vec<u16> = parse_program(&prog_string);
    println!("rom is sized {}", rom.len());

    // INIT RAM/VGA
    print!("Initializing internal data ... ");
    let mut ram: Vec<u16> = vec![0; RAM_SIZE];
    let mut vga: Vec<u16> = vec![0; VGA_SIZE];

    let mut mem: Memory = Memory { rom: &mut rom, ram: &mut ram, vga: &mut vga };

    // INIT REGISTERS
    let mut registers: Registers = Registers { general: [0; 12], isr: 0, sp: 0x8000, sr: StatusRegister { sr: 0 }, pc: 0x0000 };

    let mut halt: bool = false;
    println!("initialized!");

    while !halt {
        let pc = registers.pc;
        println!("{pc:#06x}");
        let inst: u16 = mem.mem_read(registers.pc);
        let opcode: u16 = (inst & 0xF000) >> 12;

        let r1: u16 = (inst & 0x0F00) >> 8;
        let r2: u16 = (inst & 0x00F0) >> 4;
        let imov_imm8: u16 = (inst & 0x00FF) | (if (inst & 0x0080) == 0 { 0x0000 } else { 0xFF00 });

        let alu_imm4: u16 = r2;
        let alu_op: u16 = inst & 0x000F;
        let jmp_op: u16 = alu_op;

        match opcode {
            LOAD => {
                registers[r1] = mem.mem_read(registers[r2]);
            },
            STR => {
                mem.mem_write(registers[r1], registers[r2]);
            },
            IMOV => {
                println!("r{r1}={imov_imm8:#06x}");
                registers[r1] = imov_imm8;
            },
            PUSH => {
                mem.mem_write(registers[r1], registers[r2]);
                registers.sp += 1;
            },
            POP => {
                registers.sp -= 1;
                registers[r1] = mem.mem_read(registers[r2]);
            },
            HALT => {
                halt = true;
            },
            ALU => {
                let agg = alu(alu_op, 
                    registers[r1] as i32, 
                    registers[r2] as i32, 
                    &mut registers.sr);
                
                registers[r1] = agg;
            },
            IALU => {
                let agg = alu(alu_op, 
                    registers[r1] as i32, 
                    alu_imm4 as i32, 
                    &mut registers.sr);
                
                registers[r1] = agg;
            },
            JMP => {
                let l: bool = r2 & 1 != 0;
                let r: bool = r2 & 2 != 0;

                let do_jump: bool = match jmp_op {
                    0 => true,
                    1 => registers.sr.get(StatusRegisterFlag::Z),
                    2 => !registers.sr.get(StatusRegisterFlag::Z),
                    3 => registers.sr.get(StatusRegisterFlag::N),
                    4 => !registers.sr.get(StatusRegisterFlag::Z) && 
                         !registers.sr.get(StatusRegisterFlag::N),
                    _ => false,
                };

                if do_jump {
                    if r {
                        registers.sp -= 1;
                        registers.pc = mem.mem_read(registers.sp);
                    } else if l {
                        mem.mem_write(registers.sp, registers.pc + 1);
                        registers.sp += 1;

                        registers.pc = registers[r1];
                    } else {
                        registers.pc = registers[r1];
                    }
                    registers.pc -= 1;
                }
            },
            RTI => {
                registers.sp -= 1;
                registers.sr.sr = mem.mem_read(registers.sp);
                registers.sp -= 1;
                registers.pc = mem.mem_read(registers.sp);

                registers.pc -= 1;
            }
            _ => ()
        }
        registers.pc += 1;
    }

    let last_pc = registers.pc - 1;
    println!("Program halted at PC={last_pc}");
}