mod fetch_execute;
mod instructions;

use crate::{display::DisplayBuffer, keyboard::KeyboardState, memory::Memory, timer::Timers};
use std::{
    thread,
    time::{Duration, Instant},
};
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

            cycle_delay: Duration::from_millis(2),
            last_cycle: Instant::now(),
        }
    }

    pub fn run_cycle(&mut self) {
        // TODO: Implement proper clock rate
        let now = Instant::now();
        let diff = now - self.last_cycle;
        let timers_diff = self.timers.tick();

        if diff > self.cycle_delay {
            self.last_cycle = now;
            let opcode = self.fetch();
            self.execute(opcode);
        } else {
            let smallest_diff = if diff > timers_diff {
                diff
            } else {
                timers_diff
            };
            thread::sleep(smallest_diff);
        }
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
