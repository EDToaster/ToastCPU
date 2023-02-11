pub const ROM_SIZE: usize = 0x8000;
pub const RAM_SIZE: usize = 0x4000;
pub const VGA_WIDTH: usize = 100;
pub const VGA_HEIGHT: usize = 60;

pub const LOAD: u16 = 0;
pub const STR: u16 = 1;
pub const IMOV: u16 = 2;
pub const IMOH: u16 = 3;
pub const PUSH: u16 = 5;
pub const POP: u16 = 6;
pub const HALT: u16 = 7;
pub const ALU: u16 = 8;
pub const IALU: u16 = 9;
pub const JMP: u16 = 10;
pub const RTI: u16 = 12;
