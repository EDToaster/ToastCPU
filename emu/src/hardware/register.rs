use core::ops::Index;
use std::ops::IndexMut;

/// Register-file indices for the four special registers. The general-purpose
/// registers occupy indices 0..=11. These are `u16` so they can be used
/// directly with the `Index<u16>`/`IndexMut<u16>` impls below.
pub const REG_ISR: u16 = 12;
pub const REG_SP: u16 = 13;
pub const REG_SR: u16 = 14;
pub const REG_PC: u16 = 15;

pub enum StatusRegisterFlag {
    X,
    Z,
    N,
    C,
    V,
}

impl StatusRegisterFlag {
    pub fn mask(&self) -> u16 {
        match *self {
            StatusRegisterFlag::X => 0x0001,
            StatusRegisterFlag::Z => 0x0002,
            StatusRegisterFlag::N => 0x0004,
            StatusRegisterFlag::C => 0x0008,
            StatusRegisterFlag::V => 0x0010,
        }
    }

    /// Read this flag out of a raw status-register word.
    #[inline]
    pub fn read(&self, sr: u16) -> bool {
        (sr & self.mask()) != 0
    }

    /// Set or clear this flag in a raw status-register word.
    #[inline]
    pub fn write(&self, sr: &mut u16, val: bool) {
        if val {
            *sr |= self.mask();
        } else {
            *sr &= !self.mask();
        }
    }
}

/// The register file, stored as one flat array so that indexed access
/// (`registers[r]`, where `r` is a 4-bit register number decoded from an
/// instruction) is a single array index rather than a match. Register numbers
/// 12..=15 alias the special registers, matching the ISA where programs may
/// address ISR/SP/SR/PC by number.
///
///   r0..=r11 - general purpose
///   r12 - ISR   r13 - SP   r14 - SR   r15 - PC
pub struct Registers {
    pub regs: [u16; 16],
}

impl Index<u16> for Registers {
    type Output = u16;

    #[inline]
    fn index(&self, index: u16) -> &Self::Output {
        &self.regs[index as usize]
    }
}

impl IndexMut<u16> for Registers {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.regs[index as usize]
    }
}
