use std::sync::atomic::{AtomicU16, Ordering};

/// The CPU's view of the address space.
///
/// | range             | read          | write |
/// |-------------------|---------------|-------|
/// | `0x0000..=0x7FFF` | ROM           | VRAM (only the first 6000 cells exist) |
/// | `0x8000..=0xBFFF` | RAM           | RAM   |
/// | `0xFFFF`          | last key code | -     |
pub struct Devices<'a> {
    rom: Vec<u16>,
    vram: &'a [AtomicU16],
    ram: Vec<u16>,
    key: &'a AtomicU16,
}

impl<'a> Devices<'a> {
    pub fn new(rom: Vec<u16>, vram: &'a [AtomicU16], ram: Vec<u16>, key: &'a AtomicU16) -> Devices<'a> {
        Devices { rom, vram, ram, key }
    }

    pub fn read(&self, addr: u16) -> Result<u16, String> {
        match addr {
            0..=0x7FFF => Ok(self.rom[addr as usize]),
            0x8000..=0xBFFF => Ok(self.ram[(addr - 0x8000) as usize]),
            0xFFFF => Ok(self.key.load(Ordering::Relaxed)),
            _ => Err(format!("Memory location {addr:#06x} not implemented")),
        }
    }

    pub fn write(&mut self, addr: u16, val: u16) -> Result<(), String> {
        match addr {
            0..=0x7FFF => self
                .vram
                .get(addr as usize)
                .ok_or_else(|| format!("VRAM write out of range: {addr:#06x}={val:#06x}"))?
                .store(val, Ordering::Relaxed),
            0x8000..=0xBFFF => self.ram[(addr - 0x8000) as usize] = val,
            _ => return Err(format!("Memory location {addr:#06x}={val:#06x}")),
        }
        Ok(())
    }
}
