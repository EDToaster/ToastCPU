//! A CPU runner that can be paused, stepped, reset and stopped at
//! breakpoints by a front-end.
//!
//! The CPU thread owns all machine state. The front-end talks to it through a
//! command channel and reads back a [`Snapshot`] (registers, RAM, run state)
//! that the CPU publishes on request, so the hot loop never takes a lock while
//! it is executing.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::cpu::Cpu;
use super::disasm::is_call;
use crate::devices::Devices;
use crate::hardware::def::RAM_SIZE;
use crate::hardware::register::{REG_PC, REG_SP};
use crate::io::SharedIo;

/// Instructions executed between command/shutdown checks at full speed.
const CHUNK: u64 = 1 << 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PauseReason {
    Startup,
    User,
    Step,
    Breakpoint(u16),
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunState {
    Running,
    Paused(PauseReason),
    Halted,
    Error(String),
}

impl RunState {
    pub fn is_running(&self) -> bool {
        matches!(self, RunState::Running)
    }
    /// The CPU is stopped but can execute more instructions.
    pub fn can_step(&self) -> bool {
        matches!(self, RunState::Paused(_))
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Pause,
    Resume,
    /// Execute this many instructions, then pause.
    Step(u32),
    /// Like `Step(1)`, but runs a `jmpl` call until it returns.
    StepOver,
    /// Resume until PC reaches this address.
    RunTo(u16),
    Reset,
    ToggleBreakpoint(u16),
    ClearBreakpoints,
    /// Throttle to roughly this many instructions per second (`None` = full speed).
    SetSpeed(Option<u64>),
    SetRegister(u16, u16),
}

/// Machine state as last published by the CPU thread.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub state: RunState,
    pub regs: [u16; 16],
    pub ram: Vec<u16>,
    /// Instructions retired since the last reset.
    pub retired: u64,
    pub breakpoints: BTreeSet<u16>,
    pub speed: Option<u64>,
    pub irq_pending: bool,
    pub key: u16,
}

/// Shared between the CPU thread and the front-end.
pub struct DebugLink {
    commands: Mutex<Option<Sender<Command>>>,
    snapshot: Mutex<Snapshot>,
    want_snapshot: AtomicBool,
}

impl DebugLink {
    pub fn new() -> (DebugLink, Receiver<Command>) {
        let (tx, rx) = channel();
        let link = DebugLink {
            commands: Mutex::new(Some(tx)),
            snapshot: Mutex::new(Snapshot {
                state: RunState::Running,
                regs: Cpu::new().registers.regs,
                ram: vec![0; RAM_SIZE],
                retired: 0,
                breakpoints: BTreeSet::new(),
                speed: None,
                irq_pending: false,
                key: 0,
            }),
            want_snapshot: AtomicBool::new(true),
        };
        (link, rx)
    }

    pub fn send(&self, cmd: Command) {
        if let Some(tx) = self.commands.lock().unwrap().as_ref() {
            let _ = tx.send(cmd);
        }
    }

    /// Ask the CPU thread to publish a fresh snapshot soon.
    pub fn request_snapshot(&self) {
        self.want_snapshot.store(true, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().unwrap().clone()
    }

    /// Drop the sender so a CPU thread blocked on commands notices shutdown.
    pub fn close(&self) {
        self.commands.lock().unwrap().take();
    }
}

/// 64k-entry bitset for O(1) breakpoint checks in the hot loop.
struct Breakpoints {
    bits: Vec<u64>,
    set: BTreeSet<u16>,
}

impl Breakpoints {
    fn new() -> Self {
        Breakpoints { bits: vec![0; 1024], set: BTreeSet::new() }
    }
    fn toggle(&mut self, addr: u16) {
        if !self.set.remove(&addr) {
            self.set.insert(addr);
        }
        self.bits[addr as usize / 64] ^= 1 << (addr % 64);
    }
    fn clear(&mut self) {
        self.bits.iter_mut().for_each(|b| *b = 0);
        self.set.clear();
    }
    #[inline]
    fn contains(&self, addr: u16) -> bool {
        self.bits[addr as usize / 64] & (1 << (addr % 64)) != 0
    }
}

/// A one-shot stop condition for "run to" and "step over".
#[derive(Clone, Copy)]
struct Target {
    pc: u16,
    /// Only stop when SP is back at this value (step over a call).
    sp: Option<u16>,
}

pub struct DebugRunner<'a> {
    cpu: Cpu,
    mem: Devices<'a>,
    io: &'a SharedIo,
    link: &'a DebugLink,
    state: RunState,
    breakpoints: Breakpoints,
    target: Option<Target>,
    speed: Option<u64>,
    retired: u64,
    /// For throttling: when and at what retired count the current speed
    /// window started.
    pace_start: (Instant, u64),
}

impl<'a> DebugRunner<'a> {
    pub fn new(mem: Devices<'a>, io: &'a SharedIo, link: &'a DebugLink) -> DebugRunner<'a> {
        DebugRunner {
            cpu: Cpu::new(),
            mem,
            io,
            link,
            state: RunState::Running,
            breakpoints: Breakpoints::new(),
            target: None,
            speed: None,
            retired: 0,
            pace_start: (Instant::now(), 0),
        }
    }

    pub fn start_paused(&mut self) {
        self.state = RunState::Paused(PauseReason::Startup);
    }

    pub fn add_breakpoint(&mut self, addr: u16) {
        if !self.breakpoints.contains(addr) {
            self.breakpoints.toggle(addr);
        }
    }

    fn publish(&self) {
        let mut snap = self.link.snapshot.lock().unwrap();
        snap.state = self.state.clone();
        snap.regs = self.cpu.registers.regs;
        snap.ram.copy_from_slice(self.mem.ram());
        snap.retired = self.retired;
        snap.breakpoints = self.breakpoints.set.clone();
        snap.speed = self.speed;
        snap.irq_pending = self.io.irq.load(Ordering::Relaxed);
        snap.key = self.io.key.load(Ordering::Relaxed);
    }

    fn pause(&mut self, reason: PauseReason) {
        if self.state.is_running() || self.state.can_step() {
            self.state = RunState::Paused(reason);
        }
        self.target = None;
    }

    fn resume(&mut self) {
        if self.state.can_step() {
            self.state = RunState::Running;
            self.pace_start = (Instant::now(), self.retired);
        }
    }

    /// Execute one instruction (or take an interrupt). Returns false if the
    /// machine stopped (halt or error).
    fn step_one(&mut self) -> bool {
        match self.cpu.step(&mut self.mem, self.io) {
            Ok(retired) => {
                self.retired += retired as u64;
                if self.cpu.halted {
                    self.state = RunState::Halted;
                    return false;
                }
                true
            }
            Err(e) => {
                self.state = RunState::Error(e);
                false
            }
        }
    }

    fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::Pause => self.pause(PauseReason::User),
            Command::Resume => self.resume(),
            Command::Step(n) => {
                if self.state.can_step() {
                    for i in 0..n {
                        if i > 0 && self.breakpoints.contains(self.cpu.registers[REG_PC]) {
                            self.state = RunState::Paused(PauseReason::Breakpoint(self.cpu.registers[REG_PC]));
                            return;
                        }
                        if !self.step_one() {
                            return;
                        }
                    }
                    self.state = RunState::Paused(PauseReason::Step);
                }
            }
            Command::StepOver => {
                if !self.state.can_step() {
                    return;
                }
                let pc = self.cpu.registers[REG_PC];
                let inst = self.mem.read(pc).unwrap_or(0);
                if is_call(inst) {
                    let sp = self.cpu.registers[REG_SP];
                    self.target = Some(Target { pc: pc.wrapping_add(1), sp: Some(sp) });
                    self.resume();
                } else {
                    self.handle(Command::Step(1));
                }
            }
            Command::RunTo(addr) => {
                if self.state.can_step() {
                    self.target = Some(Target { pc: addr, sp: None });
                    self.resume();
                }
            }
            Command::Reset => {
                self.cpu = Cpu::new();
                self.mem.reset();
                self.io.irq.store(false, Ordering::Relaxed);
                self.io.key.store(0, Ordering::Relaxed);
                self.retired = 0;
                self.target = None;
                self.state = if self.state.can_step() {
                    RunState::Paused(PauseReason::Reset)
                } else {
                    RunState::Running
                };
                self.pace_start = (Instant::now(), 0);
            }
            Command::ToggleBreakpoint(addr) => self.breakpoints.toggle(addr),
            Command::ClearBreakpoints => self.breakpoints.clear(),
            Command::SetSpeed(speed) => {
                self.speed = speed.filter(|&s| s > 0);
                self.pace_start = (Instant::now(), self.retired);
            }
            Command::SetRegister(r, v) => {
                if !self.state.is_running() {
                    self.cpu.registers[r & 0xF] = v;
                }
            }
        }
    }

    /// Run up to `budget` instructions, stopping early at breakpoints/targets.
    fn run_chunk(&mut self, budget: u64) {
        let check = !self.breakpoints.set.is_empty() || self.target.is_some();
        let start = self.retired;
        for i in 0..budget {
            if check && i > 0 {
                let pc = self.cpu.registers[REG_PC];
                if let Some(t) = self.target {
                    if t.pc == pc && t.sp.is_none_or(|sp| sp == self.cpu.registers[REG_SP]) {
                        self.state = RunState::Paused(PauseReason::Step);
                        self.target = None;
                        break;
                    }
                }
                if self.breakpoints.contains(pc) {
                    self.state = RunState::Paused(PauseReason::Breakpoint(pc));
                    self.target = None;
                    break;
                }
            }
            if !self.step_one() {
                break;
            }
        }
        self.io.running_count.fetch_add(self.retired - start, Ordering::Relaxed);
    }

    /// Main loop for the CPU thread. Returns when the front-end shuts down;
    /// the result is the last error the program hit, if any.
    pub fn run(mut self, commands: Receiver<Command>) -> Result<(), String> {
        self.publish();
        loop {
            if self.io.should_terminate() {
                break;
            }

            // Commands: block (briefly) while stopped, poll while running.
            if self.state.is_running() {
                while let Ok(cmd) = commands.try_recv() {
                    self.handle(cmd);
                }
            } else {
                self.publish();
                match commands.recv_timeout(Duration::from_millis(50)) {
                    Ok(cmd) => {
                        self.handle(cmd);
                        while let Ok(cmd) = commands.try_recv() {
                            self.handle(cmd);
                        }
                        self.publish();
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }

            if self.state.is_running() {
                match self.speed {
                    None => self.run_chunk(CHUNK),
                    Some(ips) => {
                        // Run whatever the target rate allows so far, in
                        // small slices so pause/step stay responsive.
                        let (t0, n0) = self.pace_start;
                        let allowed = (t0.elapsed().as_secs_f64() * ips as f64) as u64;
                        let done = self.retired - n0;
                        if allowed > done {
                            self.run_chunk((allowed - done).min(CHUNK));
                        } else {
                            std::thread::sleep(Duration::from_millis(2));
                        }
                    }
                }
                if !self.state.is_running() {
                    self.publish();
                }
            }

            if self.link.want_snapshot.swap(false, Ordering::Relaxed) {
                self.publish();
            }
        }

        match self.state {
            RunState::Error(e) => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::def::ROM_SIZE;

    /// A tiny loop: r1 = 0; loop: r1 += 1; jmp loop
    fn counting_rom() -> Vec<u16> {
        let mut rom = vec![0x7000; ROM_SIZE];
        rom[0] = 0x2100; // imov p0 0
        rom[1] = 0x2202; // imov p1 2       (loop address)
        rom[2] = 0x9114; // iadd p0 1
        rom[3] = 0xA200; // jmp p1
        rom
    }

    fn with_runner(f: impl FnOnce(&mut DebugRunner, &Receiver<Command>)) {
        let io = SharedIo::new();
        let (link, rx) = DebugLink::new();
        let mem = Devices::new(counting_rom(), &io.vram, vec![0; RAM_SIZE], &io.key);
        let mut r = DebugRunner::new(mem, &io, &link);
        r.start_paused();
        f(&mut r, &rx);
    }

    #[test]
    fn steps_and_breakpoints() {
        with_runner(|r, _| {
            r.handle(Command::Step(3));
            assert_eq!(r.cpu.registers[REG_PC], 3);
            assert_eq!(r.cpu.registers[1], 1);
            assert_eq!(r.state, RunState::Paused(PauseReason::Step));

            r.handle(Command::ToggleBreakpoint(2));
            r.handle(Command::Resume);
            r.run_chunk(1000);
            assert_eq!(r.state, RunState::Paused(PauseReason::Breakpoint(2)));
            assert_eq!(r.cpu.registers[1], 1);

            // resuming from a breakpoint executes it instead of re-hitting it
            r.handle(Command::Resume);
            r.run_chunk(1000);
            assert_eq!(r.state, RunState::Paused(PauseReason::Breakpoint(2)));
            assert_eq!(r.cpu.registers[1], 2);

            r.handle(Command::ToggleBreakpoint(2));
            r.handle(Command::Resume);
            r.run_chunk(1000);
            assert!(r.state.is_running());
        });
    }

    #[test]
    fn run_to_and_reset() {
        with_runner(|r, _| {
            r.handle(Command::RunTo(3));
            r.run_chunk(1000);
            assert_eq!(r.cpu.registers[REG_PC], 3);
            assert_eq!(r.state, RunState::Paused(PauseReason::Step));

            r.handle(Command::SetRegister(1, 0x1234));
            assert_eq!(r.cpu.registers[1], 0x1234);

            r.handle(Command::Reset);
            assert_eq!(r.cpu.registers[REG_PC], 0);
            assert_eq!(r.cpu.registers[1], 0);
            assert_eq!(r.retired, 0);
            assert_eq!(r.state, RunState::Paused(PauseReason::Reset));
        });
    }

    #[test]
    fn throttles_and_shuts_down() {
        let io = SharedIo::new();
        let (link, rx) = DebugLink::new();
        let mem = Devices::new(counting_rom(), &io.vram, vec![0; RAM_SIZE], &io.key);
        let runner = DebugRunner::new(mem, &io, &link);
        link.send(Command::SetSpeed(Some(1000)));
        std::thread::scope(|s| {
            let h = s.spawn(|| runner.run(rx));
            std::thread::sleep(Duration::from_millis(300));
            link.send(Command::Pause);
            std::thread::sleep(Duration::from_millis(100));
            let snap = link.snapshot();
            assert!(snap.state.can_step(), "{:?}", snap.state);
            // ~300 instructions at 1k ips; allow generous scheduling slack
            assert!((100..=600).contains(&snap.retired), "retired {}", snap.retired);
            io.terminate();
            link.close();
            assert!(h.join().unwrap().is_ok());
        });
    }

    #[test]
    fn halts() {
        with_runner(|r, _| {
            let mut rom = vec![0x7000; ROM_SIZE];
            rom[0] = 0x2105;
            r.mem = Devices::new(rom, &r.io.vram, vec![0; RAM_SIZE], &r.io.key);
            r.handle(Command::Resume);
            r.run_chunk(10);
            assert_eq!(r.state, RunState::Halted);
            r.handle(Command::Step(1));
            assert_eq!(r.state, RunState::Halted);
        });
    }
}
