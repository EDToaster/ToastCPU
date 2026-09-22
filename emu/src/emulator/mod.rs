use std::io::stdout;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;

use crate::devices::Devices;
use crate::hardware::def::*;
use crate::hardware::key::Key;
use crate::hardware::register::{Registers, StatusRegisterFlag, REG_ISR, REG_PC, REG_SP, REG_SR};
use crate::hardware::vga::Vga;

/// How many instructions to count locally before publishing the total to the
/// shared `running_count` atomic. Keeps the per-instruction cost to a plain
/// local increment instead of a locked atomic RMW; the VGA thread only samples
/// the count every ~100ms for its diagnostics, so this granularity is ample.
const COUNT_BATCH: u64 = 1 << 16;

fn bit(n: i32, bit: u8) -> bool {
    (n >> bit) & 1 != 0
}

fn alu(op: u16, a: i32, b: i32, sr: &mut u16) -> u16 {
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

    StatusRegisterFlag::N.write(sr, agg & 0x8000 != 0);
    StatusRegisterFlag::Z.write(sr, agg == 0);
    StatusRegisterFlag::X.write(sr, agg == 0xFFFF);

    match op {
        0x4 => {
            StatusRegisterFlag::V.write(
                sr,
                (bit(a, 15) == bit(b, 15)) && (bit(a, 15) ^ bit(agg, 15)),
            );
            StatusRegisterFlag::C.write(sr, bit(agg, 16));
        }
        0x5 | 0x7 => {
            StatusRegisterFlag::V.write(
                sr,
                (bit(a, 15) ^ bit(b, 15)) && (bit(a, 15) != bit(agg, 15)),
            );
            StatusRegisterFlag::C.write(sr, bit(agg, 16));
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

    let key: AtomicU16 = AtomicU16::new(0);
    let irq: AtomicBool = AtomicBool::new(false);

    // INIT REGISTERS
    let mut registers: Registers = Registers { regs: [0u16; 16] };
    registers[REG_SP] = 0xBFFF;

    let mut halt: bool = false;

    let mut key_handler = Key::new(&irq, &key);

    let term = AtomicBool::new(false);
    let term1 = &term;
    let term2 = &term;

    std::thread::scope(|scope| -> Result<(), String> {
        // start keyboard thread
        let key_handler_thread = scope.spawn(move || key_handler.handle(term1));
        // start vga thread
        let display_thread = scope.spawn(move || disp_vga.start_loop(term2));

        // main thread
        let mut mem = Devices::new(rom, &vram, ram, &key);
        let mut local_count: u64 = 0;
        while !halt {
            if irq.load(Ordering::Acquire) {
                irq.store(false, Ordering::Relaxed);
                registers[REG_SP] -= 1;
                mem.write(registers[REG_SP], registers[REG_PC]).map_err(|e| {
                    format!(
                        "Issue when jumping to isr storing stack pointer instruction pc={:#06x}: {e}",
                        registers[REG_PC]
                    )
                })?;
                registers[REG_SP] -= 1;
                mem.write(registers[REG_SP], registers[REG_SR]).map_err(|e| {
                    format!(
                        "Issue when jumping to isr storing status register instruction pc={:#06x}: {e}",
                        registers[REG_PC]
                    )
                })?;
                registers[REG_PC] = registers[REG_ISR];
                continue;
            }

            let inst: u16 = mem.read(registers[REG_PC]).map_err(|e| {
                format!("Issue when read instruction pc={:#06x}: {e}", registers[REG_PC])
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
                        format!("Issue when executing load at pc={:#06x}: {e}", registers[REG_PC])
                    })?;
                }
                STR => {
                    mem.write(registers[r1] + load_offset, registers[r2])
                        .map_err(|e| {
                            format!(
                                "Issue when executing store at pc={:#06x}: {e}",
                                registers[REG_PC]
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
                        format!("Issue when executing push at pc={:#06x}: {e}", registers[REG_PC])
                    })?;
                }
                POP => {
                    registers[r1] = mem.read(registers[r2]).map_err(|e| {
                        format!("Issue when executing pop at pc={:#06x}: {e}", registers[REG_PC])
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
                        &mut registers[REG_SR],
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
                        &mut registers[REG_SR],
                    );

                    if alu_op != 0x7 {
                        registers[r1] = agg;
                    }
                }
                JMP => {
                    let l: bool = r2 & 1 != 0;
                    let r: bool = r2 & 2 != 0;

                    let sr = registers[REG_SR];
                    let do_jump: bool = match jmp_op {
                        0 => true,
                        1 => StatusRegisterFlag::Z.read(sr),
                        2 => !StatusRegisterFlag::Z.read(sr),
                        3 => StatusRegisterFlag::N.read(sr),
                        4 => !StatusRegisterFlag::Z.read(sr) && !StatusRegisterFlag::N.read(sr),
                        _ => false,
                    };

                    if do_jump {
                        if r {
                            registers[REG_PC] = mem.read(registers[REG_SP]).map_err(|e| {
                                format!(
                                    "Issue when executing jump at pc={:#06x}: {e}",
                                    registers[REG_PC]
                                )
                            })?;
                            registers[REG_SP] += 1;
                        } else if l {
                            registers[REG_SP] -= 1;
                            mem.write(registers[REG_SP], registers[REG_PC] + 1).map_err(|e| {
                                format!(
                                    "Issue when executing jump and link at pc={:#06x}: {e}",
                                    registers[REG_PC]
                                )
                            })?;
                            registers[REG_PC] = registers[r1];
                        } else {
                            registers[REG_PC] = registers[r1];
                        }
                        registers[REG_PC] -= 1;
                    }
                }
                RTI => {
                    registers[REG_SR] = mem.read(registers[REG_SP])
                        .map_err(|e| format!("Issue when executing rti and popping the status register at pc={:#06x}: {e}", registers[REG_PC]))?;
                    registers[REG_SP] += 1;
                    registers[REG_PC] = mem.read(registers[REG_SP])
                        .map_err(|e| format!("Issue when executing rti and popping the return address at pc={:#06x}: {e}", registers[REG_PC]))?;
                    registers[REG_SP] += 1;
                    registers[REG_PC] -= 1;
                }
                _ => (),
            }
            registers[REG_PC] += 1;
            local_count += 1;
            if local_count >= COUNT_BATCH {
                running_count.fetch_add(local_count, Ordering::Relaxed);
                local_count = 0;
            }
        }

        // Publish any instructions counted since the last batch so the final
        // diagnostics frame isn't missing them.
        running_count.fetch_add(local_count, Ordering::Relaxed);

        let last_pc = registers[REG_PC] - 1;

        term.swap(true, Ordering::Relaxed);

        key_handler_thread.join().unwrap();
        display_thread.join().unwrap();

        Ok(())
    })?;

    execute!(stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
    Ok(())
}
