use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::vga::Vga;

pub struct Diagnostics {
    interval: Duration,
    prev_time: SystemTime,
    running_count: i64,

    vga: Arc<Mutex<Vga>>,
}

impl Diagnostics {
    pub fn new(vga: Arc<Mutex<Vga>>, interval: Duration) -> Diagnostics {
        Diagnostics {
            interval,
            prev_time: SystemTime::now(),
            vga,
            running_count: 0,
        }
    }

    pub fn increment(&mut self) {
        let now: SystemTime = SystemTime::now();
        self.running_count += 1;

        let duration = now
            .duration_since(self.prev_time)
            .expect("Time went backwards!");
        if duration >= self.interval {
            let per_second = (self.running_count as f64 / duration.as_secs_f64()) as i64;
            self.prev_time = now;
            self.running_count = 0;

            let count = self.vga.lock().unwrap().write_count;

            self.vga
                .lock()
                .unwrap()
                .put_diagnostics(0, format!("{per_second} i/s {count}").as_str());
        }
    }

    pub fn halt(&mut self, pc: u16) {
        self.vga
            .lock()
            .unwrap()
            .put_diagnostics(0, format!("Halted at {pc:04x}").as_str());
    }
}
