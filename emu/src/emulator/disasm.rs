//! Disassembler for single `toast` instruction words (see `spec/arch.md`).

use crate::hardware::def::*;

/// ABI names for r0..r15 (see `spec/abi.md`).
pub const REG_NAMES: [&str; 16] = [
    "ar", "p0", "p1", "p2", "p3", "v0", "t0", "t1", "t2", "t3", "t4", "t5", "isr", "sp", "sr", "pc",
];

const ALU_OPS: [&str; 11] = ["not", "and", "or", "xor", "add", "sub", "mov", "cmp", "shr", "sshr", "shl"];
const JMP_OPS: [&str; 5] = ["jmp", "jz", "jnz", "jn", "jp"];

fn reg(r: u16) -> &'static str {
    REG_NAMES[r as usize & 0xF]
}

/// Is this a `jmpl`-style instruction (pushes a return address)?
pub fn is_call(inst: u16) -> bool {
    (inst >> 12) == JMP && (inst >> 4) & 0b11 == 0b01
}

pub fn disassemble(inst: u16) -> String {
    let opcode = inst >> 12;
    let r1 = (inst >> 8) & 0xF;
    let r2 = (inst >> 4) & 0xF;
    let low = inst & 0xF;
    let imm8 = inst & 0xFF;

    let with_offset = |s: String| if low == 0 { s } else { format!("{s} +{low}") };

    match opcode {
        LOAD => with_offset(format!("load  {} {}", reg(r1), reg(r2))),
        STR => with_offset(format!("str   {} {}", reg(r1), reg(r2))),
        IMOV => {
            let v = imm8 | if imm8 & 0x80 != 0 { 0xFF00 } else { 0 };
            format!("imov  {} 0x{v:04X}", reg(r1))
        }
        IMOH => format!("imoh  {} 0x{imm8:02X}", reg(r1)),
        PUSH => format!("push  {} {}", reg(r1), reg(r2)),
        POP => format!("pop   {} {}", reg(r1), reg(r2)),
        HALT => "halt".to_string(),
        ALU | IALU => {
            let i = if opcode == IALU { "i" } else { "" };
            match ALU_OPS.get(low as usize) {
                Some(&"not") => format!("{i}not  {}", reg(r1)),
                Some(op) if opcode == IALU => format!("{i}{op:<4} {} {r2}", reg(r1)),
                Some(op) => format!("{op:<5} {} {}", reg(r1), reg(r2)),
                None => format!(".word 0x{inst:04X}"),
            }
        }
        JMP => {
            let Some(base) = JMP_OPS.get(low as usize) else {
                return format!(".word 0x{inst:04X}");
            };
            match r2 & 0b11 {
                0b10 | 0b11 => format!("{base}r"),
                0b01 => format!("{base}l {}", reg(r1)),
                _ => format!("{base} {}", reg(r1)),
            }
        }
        RTI => "rti".to_string(),
        _ => format!(".word 0x{inst:04X}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disassembles_each_format() {
        assert_eq!(disassemble(0x2C47), "imov  isr 0x0047");
        assert_eq!(disassemble(0x26FF), "imov  t0 0xFFFF");
        assert_eq!(disassemble(0x3C02), "imoh  isr 0x02");
        assert_eq!(disassemble(0x8B64), "add   t5 t0");
        assert_eq!(disassemble(0x9637), "icmp  t0 3");
        assert_eq!(disassemble(0x8600), "not  t0");
        assert_eq!(disassemble(0x5B70), "push  t5 t1");
        assert_eq!(disassemble(0x66D0), "pop   t0 sp");
        assert_eq!(disassemble(0x0675), "load  t0 t1 +5");
        assert_eq!(disassemble(0xA010), "jmpl ar");
        assert_eq!(disassemble(0xA021), "jzr");
        assert_eq!(disassemble(0xA002), "jnz ar");
        assert_eq!(disassemble(0x7000), "halt");
        assert_eq!(disassemble(0xC000), "rti");
        assert_eq!(disassemble(0x4123), ".word 0x4123");
        assert!(is_call(0xA010));
        assert!(!is_call(0xA000));
    }
}
