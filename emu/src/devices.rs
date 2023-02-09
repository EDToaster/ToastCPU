use std::sync::{Arc, Mutex, atomic::{AtomicU16, Ordering}};

pub struct Devices<'a> {
    rom: Vec<u16>,
    vram: &'a Vec<AtomicU16>,
    ram: Vec<u16>,
    key: Arc<Mutex<u16>>,
}

impl <'a> Devices<'a> {
    pub fn new(
        rom: Vec<u16>,
        vram: &Vec<AtomicU16>,
        ram: Vec<u16>,
        key: Arc<Mutex<u16>>,
    ) -> Devices {
        Devices { rom, vram, ram, key }
    }

    pub fn read(&self, addr: u16) -> Result<u16, String> {
        match addr {
            0..=0x7FFF => Ok(self.rom[addr as usize]),
            0x8000..=0xBFFF => Ok(self.ram[(addr - 0x8000) as usize]),
            0xFFFF => {
                let a = *self.key.lock().unwrap();
                Ok(a)
            }
            _ => Err(format!("Memory location {addr:#06x} not implemented")),
        }
    }

    pub fn write(&mut self, addr: u16, val: u16) -> Result<(), String> {
        match addr {
            0..=0x7FFF => {
                self.vram[addr as usize].swap(val, Ordering::Relaxed);
            }
            0x8000..=0xBFFF => self.ram[(addr - 0x8000) as usize] = val,
            _ => return Err(format!("Memory location {addr:#06x}={val:#06x}")),
        }
        Ok(())
    }
}
