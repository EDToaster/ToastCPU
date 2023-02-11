
use core::ops::Index;
use std::ops::IndexMut;

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
}

/**
   if (set_VC) SR[4:3] <= {V, C};
   SR[2:0] <= {N, Z, X};
*/
pub struct StatusRegister {
    pub sr: u16,
}

impl StatusRegister {
    pub fn get(&self, flag: StatusRegisterFlag) -> bool {
        (self.sr & flag.mask()) != 0
    }

    pub fn set(&mut self, flag: StatusRegisterFlag, val: bool) {
        if val {
            self.sr |= flag.mask();
        } else {
            self.sr &= !flag.mask();
        }
    }
}
pub struct Registers {
    pub general: [u16; 12],
    // # r12 - ISR
    // # r13 - SP
    // # r14 - SR
    // # r15 - PC
    pub isr: u16,
    pub sp: u16,
    pub sr: StatusRegister,
    pub pc: u16,
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
