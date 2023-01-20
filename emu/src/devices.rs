use std::{rc::Rc, sync::{Arc, Mutex}};

use crate::vga::Vga;

pub struct Devices {
    rom: Rc<Vec<u16>>,
    vga: Arc<Mutex<Vga>>,
    ram: Rc<Vec<u16>>,
    key: Arc<Mutex<u16>>,
}

impl Devices {
    pub fn new(
        rom: Rc<Vec<u16>>,
        vga: Arc<Mutex<Vga>>,
        ram: Rc<Vec<u16>>,
        key: Arc<Mutex<u16>>,
    ) -> Devices {
        Devices { rom, vga, ram, key }
    }

    pub fn read(&self, addr: u16) -> Result<u16, String> {
        match addr {
            0..=0x7FFF => Ok(self.rom[addr as usize]),
            0x8000..=0xBFFF => Ok(self.ram[(addr - 0x8000) as usize]),
            0xFFFF => {
                let a = *self.key.lock().unwrap();
                // println!("{:04x} at {:04x}", a, addr);
                Ok(a)
            },
            _ => Err(format!("Memory location {addr:#06x} not implemented")),
        }
    }

    pub fn write(&mut self, addr: u16, val: u16) -> Result<(), String> {
        match addr {
            0..=0x7FFF => {
                // println!("{:04x} at {:04x}", val, addr);
                self.vga.lock().unwrap().put_char(addr.into(), val);
            },
            0x8000..=0xBFFF => Rc::get_mut(&mut self.ram).unwrap()[(addr - 0x8000) as usize] = val,
            _ => return Err(format!("Memory location {addr:#06x}={val:#06x}")),
        }
        Ok(())
    }
}
