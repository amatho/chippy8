mod fetch_execute;
mod instructions;

use crate::{display::DisplayBuffer, keyboard::KeyboardState, memory::Memory, timer::Timers};
use fetch_execute::{Executor, Fetcher};
use std::time::{Duration, Instant};
use winit::event::VirtualKeyCode;

pub struct Interpreter {
    memory: Memory,
    display_buf: DisplayBuffer,
    timers: Timers,
    keyboard_state: KeyboardState,

    stack: Vec<u16>,
    program_counter: usize,
    reg_i: u16,
    reg_v: [u8; 16],

    cycle_delay: Duration,
    last_cycle: Instant,
}

impl Interpreter {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_rom(rom);

        Interpreter {
            memory,
            display_buf: DisplayBuffer::new(),
            timers: Timers::new(),
            keyboard_state: KeyboardState::new(),

            stack: Vec::with_capacity(16),
            program_counter: 0x200,
            reg_i: 0,
            reg_v: [0; 16],

            cycle_delay: Duration::from_millis(1),
            last_cycle: Instant::now(),
        }
    }

    pub fn run_cycle(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cycle) < self.cycle_delay {
            return;
        }
        self.last_cycle = now;

        let opcode = Fetcher::fetch(self);
        Executor::execute(self, opcode);

        self.timers.tick();
    }

    pub fn get_display_buffer(&self) -> &[bool; DisplayBuffer::SIZE] {
        self.display_buf.buffer()
    }

    pub fn handle_input(&mut self, key_code: VirtualKeyCode, pressed: bool) {
        self.keyboard_state.handle_input(key_code, pressed);
    }

    /// Returns a copy of the value in register `v`.
    fn reg_v(&self, index: u8) -> u8 {
        self.reg_v[index as usize]
    }

    /// Returns a mutable reference to the value in register `v`.
    fn reg_v_mut(&mut self, index: u8) -> &mut u8 {
        self.reg_v.get_mut(index as usize).unwrap()
    }
}
