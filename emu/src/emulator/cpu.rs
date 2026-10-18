use std::sync::atomic::Ordering;

use crate::devices::Devices;
use crate::hardware::def::*;
use crate::hardware::register::{Registers, StatusRegisterFlag, REG_ISR, REG_PC, REG_SP, REG_SR};
use crate::io::SharedIo;

/// How many instructions to count locally before publishing the total to the
/// shared `running_count` atomic. Keeps the per-instruction cost to a plain
/// local increment instead of a locked atomic RMW; the front-ends only sample
/// the count every ~100ms for their diagnostics, so this granularity is ample.
/// The CPU also checks for an external shutdown request at this granularity.
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
            StatusRegisterFlag::V.write(sr, (bit(a, 15) == bit(b, 15)) && (bit(a, 15) ^ bit(agg, 15)));
            StatusRegisterFlag::C.write(sr, bit(agg, 16));
        }
        0x5 | 0x7 => {
            StatusRegisterFlag::V.write(sr, (bit(a, 15) ^ bit(b, 15)) && (bit(a, 15) != bit(agg, 15)));
            StatusRegisterFlag::C.write(sr, bit(agg, 16));
        }
        _ => (),
    }

    (agg & 0xFFFF) as u16
}

/// Architectural CPU state plus the fetch/decode/execute logic.
pub struct Cpu {
    pub registers: Registers,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut registers = Registers { regs: [0u16; 16] };
        registers[REG_SP] = 0xBFFF;
        Cpu { registers, halted: false }
    }

    fn pc(&self) -> u16 {
        self.registers[REG_PC]
    }

    /// Push PC and SR and vector to the ISR held in r12.
    fn enter_isr(&mut self, mem: &mut Devices) -> Result<(), String> {
        let regs = &mut self.registers;
        regs[REG_SP] -= 1;
        mem.write(regs[REG_SP], regs[REG_PC]).map_err(|e| {
            format!("Issue when jumping to isr storing stack pointer instruction pc={:#06x}: {e}", regs[REG_PC])
        })?;
        regs[REG_SP] -= 1;
        mem.write(regs[REG_SP], regs[REG_SR]).map_err(|e| {
            format!("Issue when jumping to isr storing status register instruction pc={:#06x}: {e}", regs[REG_PC])
        })?;
        regs[REG_PC] = regs[REG_ISR];
        Ok(())
    }

    /// Execute one instruction (or take a pending interrupt).
    /// Returns `true` if an instruction was retired.
    pub fn step(&mut self, mem: &mut Devices, io: &SharedIo) -> Result<bool, String> {
        if io.irq.load(Ordering::Acquire) {
            io.irq.store(false, Ordering::Relaxed);
            self.enter_isr(mem)?;
            return Ok(false);
        }

        let pc = self.pc();
        let regs = &mut self.registers;

        let inst: u16 = mem
            .read(pc)
            .map_err(|e| format!("Issue when read instruction pc={pc:#06x}: {e}"))?;
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
                regs[r1] = mem
                    .read(regs[r2] + load_offset)
                    .map_err(|e| format!("Issue when executing load at pc={pc:#06x}: {e}"))?;
            }
            STR => {
                mem.write(regs[r1] + load_offset, regs[r2])
                    .map_err(|e| format!("Issue when executing store at pc={pc:#06x}: {e}"))?;
            }
            IMOV => regs[r1] = imov_imm8,
            IMOH => regs[r1] = imoh_imm8 | (regs[r1] & 0x00FF),
            PUSH => {
                regs[r1] -= 1;
                mem.write(regs[r1], regs[r2])
                    .map_err(|e| format!("Issue when executing push at pc={pc:#06x}: {e}"))?;
            }
            POP => {
                regs[r1] = mem
                    .read(regs[r2])
                    .map_err(|e| format!("Issue when executing pop at pc={pc:#06x}: {e}"))?;
                regs[r2] += 1;
            }
            HALT => self.halted = true,
            ALU | IALU => {
                let b = if opcode == ALU { regs[r2] } else { alu_imm4 };
                let agg = alu(alu_op, regs[r1] as i32, b as i32, &mut regs[REG_SR]);
                // 0x7 is `cmp`: flags only, no write-back.
                if alu_op != 0x7 {
                    regs[r1] = agg;
                }
            }
            JMP => {
                let l: bool = r2 & 1 != 0;
                let r: bool = r2 & 2 != 0;

                let sr = regs[REG_SR];
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
                        regs[REG_PC] = mem
                            .read(regs[REG_SP])
                            .map_err(|e| format!("Issue when executing jump at pc={pc:#06x}: {e}"))?;
                        regs[REG_SP] += 1;
                    } else if l {
                        regs[REG_SP] -= 1;
                        mem.write(regs[REG_SP], regs[REG_PC] + 1)
                            .map_err(|e| format!("Issue when executing jump and link at pc={pc:#06x}: {e}"))?;
                        regs[REG_PC] = regs[r1];
                    } else {
                        regs[REG_PC] = regs[r1];
                    }
                    // compensate for the unconditional PC increment below
                    regs[REG_PC] -= 1;
                }
            }
            RTI => {
                regs[REG_SR] = mem.read(regs[REG_SP]).map_err(|e| {
                    format!("Issue when executing rti and popping the status register at pc={pc:#06x}: {e}")
                })?;
                regs[REG_SP] += 1;
                regs[REG_PC] = mem.read(regs[REG_SP]).map_err(|e| {
                    format!("Issue when executing rti and popping the return address at pc={pc:#06x}: {e}")
                })?;
                regs[REG_SP] += 1;
                regs[REG_PC] -= 1;
            }
            _ => (),
        }
        regs[REG_PC] += 1;
        Ok(true)
    }

    /// Run until the program halts, an error occurs, or a front-end requests
    /// shutdown via `io.term`. Publishes retired-instruction counts to
    /// `io.running_count` and marks `io.halted` on exit.
    pub fn run(&mut self, mem: &mut Devices, io: &SharedIo) -> Result<(), String> {
        let mut local_count: u64 = 0;
        let result = loop {
            if self.halted {
                break Ok(());
            }
            match self.step(mem, io) {
                Ok(true) => {
                    local_count += 1;
                    if local_count >= COUNT_BATCH {
                        io.running_count.fetch_add(local_count, Ordering::Relaxed);
                        local_count = 0;
                        if io.should_terminate() {
                            break Ok(());
                        }
                    }
                }
                Ok(false) => (),
                Err(e) => break Err(e),
            }
        };
        // Publish any instructions counted since the last batch so the final
        // diagnostics frame isn't missing them.
        io.running_count.fetch_add(local_count, Ordering::Relaxed);
        if let Err(e) = &result {
            let _ = io.error.set(e.clone());
        }
        io.halted.store(true, Ordering::Release);
        result
    }
}
