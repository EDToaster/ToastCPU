use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::OnceLock;

use crate::hardware::def::VRAM_SIZE;

/// State shared between the CPU thread and the front-end (display + keyboard)
/// threads. Everything here is lock-free so the CPU's hot loop never blocks on
/// a renderer.
pub struct SharedIo {
    /// Text-mode framebuffer. Each cell is `0b00FF_FBBB_CCCC_CCCC`
    /// (fg colour, bg colour, character), as in `fpga/src/drivers/vga_driver.sv`.
    pub vram: Vec<AtomicU16>,
    /// Last scan code delivered by the keyboard, readable at `0xFFFF`.
    pub key: AtomicU16,
    /// Raised by the keyboard front-end; the CPU services it before its next instruction.
    pub irq: AtomicBool,
    /// Set when the emulator should shut down (CPU halted, window closed, ...).
    pub term: AtomicBool,
    /// Set by the CPU thread once it has stopped executing (halt or error).
    pub halted: AtomicBool,
    /// Instructions retired since the front-end last sampled it.
    pub running_count: AtomicU64,
    /// The error that stopped the CPU, if any, so a front-end can display it.
    pub error: OnceLock<String>,
}

impl SharedIo {
    pub fn new() -> SharedIo {
        SharedIo {
            vram: (0..VRAM_SIZE).map(|_| AtomicU16::new(0)).collect(),
            key: AtomicU16::new(0),
            irq: AtomicBool::new(false),
            term: AtomicBool::new(false),
            halted: AtomicBool::new(false),
            running_count: AtomicU64::new(0),
            error: OnceLock::new(),
        }
    }

    /// Deliver a key press: publish the scan code first, then raise the IRQ
    /// with Release ordering so a CPU that observes the flag (with Acquire) is
    /// guaranteed to see the key.
    pub fn press_key(&self, scan_code: u16) {
        self.key.store(scan_code, Ordering::Relaxed);
        self.irq.store(true, Ordering::Release);
    }

    pub fn should_terminate(&self) -> bool {
        self.term.load(Ordering::Relaxed)
    }

    pub fn is_halted(&self) -> bool {
        self.halted.load(Ordering::Acquire)
    }

    pub fn terminate(&self) {
        self.term.store(true, Ordering::Relaxed);
    }
}
