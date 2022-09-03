use std::{rc::Rc, sync::{Arc, Mutex}, process::exit};

use crate::vga::VGA;

pub struct Devices {
    rom: Rc<Vec<u16>>,
    vga: Arc<Mutex<VGA>>,
    ram: Rc<Vec<u16>>,
    key: Arc<Mutex<u16>>,
}

impl Devices {
    pub fn new(
        rom: Rc<Vec<u16>>,
        vga: Arc<Mutex<VGA>>,
        ram: Rc<Vec<u16>>,
        key: Arc<Mutex<u16>>,
    ) -> Devices {
        Devices { rom, vga, ram, key }
    }

    pub fn read(&self, addr: u16) -> u16 {
        match addr {
            0..=0x7FFF => self.rom[addr as usize] as u16,
            0x8000..=0xBFFF => self.ram[(addr - 0x8000) as usize] as u16,
            0xFFFF => *self.key.lock().unwrap(),
            _ => todo!("Memory location {addr:#06x}"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u16) {
        match addr {
            0..=0x7FFF => self.vga.lock().unwrap().put_char(addr.into(), val),
            0x8000..=0xBFFF => Rc::get_mut(&mut self.ram).unwrap()[(addr - 0x8000) as usize] = val,
            _ => todo!("Memory location {addr:#06x}={val:#06x}"),
        }
    }
}
