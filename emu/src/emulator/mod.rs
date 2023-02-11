use std::io::stdout;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;

use crate::devices::Devices;
use crate::hardware::def::*;
use crate::hardware::key::Key;
use crate::hardware::register::{Registers, StatusRegister, StatusRegisterFlag};
use crate::hardware::vga::Vga;

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

#[allow(unused_variables)]
pub fn emulate(rom: Vec<u16>) -> Result<(), String> {
    let ram: Vec<u16> = vec![0; RAM_SIZE];

    let vram: Vec<AtomicU16> = (0..6000).map(|_| AtomicU16::new(0)).collect();

    let running_count = AtomicU64::new(0);

    let mut disp_vga: Vga = Vga::new(
        VGA_WIDTH,
        VGA_HEIGHT,
        stdout(),
        &vram,
        Duration::new(0, 100_000_000),
        &running_count,
    );
    disp_vga.reset();

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

    let mut key_handler = Key::new(Arc::clone(&irq), Arc::clone(&key));

    let term = AtomicBool::new(false);
    let term1 = &term;
    let term2 = &term;

    std::thread::scope(|scope| -> Result<(), String> {
        // start keyboard thread
        let key_handler_thread = scope.spawn(move || key_handler.handle(term1));
        // start vga thread
        let display_thread = scope.spawn(move || disp_vga.start_loop(term2));

        // main thread
        let mut mem = Devices::new(rom, &vram, ram, key);
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

            let inst: u16 = mem.read(registers.pc).map_err(|e| {
                format!("Issue when read instruction pc={:#06x}: {e}", registers.pc)
            })?;
            let opcode: u16 = (inst & 0xF000) >> 12;

            let r1: u16 = (inst & 0x0F00) >> 8;
            let r2: u16 = (inst & 0x00F0) >> 4;
            let imoh_imm8: u16 = (inst & 0x00FF) << 8;
            let imov_imm8: u16 =
                (inst & 0x00FF) | (if (inst & 0x0080) == 0 { 0x0000 } else { 0xFF00 });

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
                                format!(
                                    "Issue when executing jump at pc={:#06x}: {e}",
                                    registers.pc
                                )
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
            running_count.fetch_add(1, Ordering::Relaxed);
        }

        let last_pc = registers.pc - 1;

        term.swap(true, Ordering::Relaxed);

        key_handler_thread.join().unwrap();
        display_thread.join().unwrap();

        Ok(())
    })?;

    execute!(stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
    Ok(())
}
