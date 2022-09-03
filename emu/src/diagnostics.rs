use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

use crate::vga::VGA;


pub struct Diagnostics {
    interval: Duration,
    prev_time: SystemTime,
    running_count: i64,

    vga: Arc<Mutex<VGA>>
}

impl Diagnostics {
    pub fn new(vga: Arc<Mutex<VGA>>, interval: Duration) -> Diagnostics {
        Diagnostics { interval, prev_time: SystemTime::now(), vga, running_count: 0 }
    }

    pub fn increment(&mut self) {
        let now: SystemTime = SystemTime::now();
        self.running_count += 1;

        let duration = now.duration_since(self.prev_time).expect("Time went backwards!");
        if duration >= self.interval {
            let per_second = (self.running_count as f64 / duration.as_secs_f64()) as i64;
            self.prev_time = now;
            self.running_count = 0;

            self.vga.lock().unwrap().put_dianostics(0, format!("{per_second} i/s").as_str());
        }
    }
}