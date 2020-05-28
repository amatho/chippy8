mod executor;

use crate::{display::DisplayBuffer, keyboard::KeyboardState, memory::Memory, timer::Timers};
use executor::{OpcodeExecutor, ProgramCounterChange};
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};
use winit::event::VirtualKeyCode;

pub struct Processor {
    memory: Memory,
    registers: Registers,
    stack: Stack,
    display_buf: DisplayBuffer,
    timers: Timers,
    keyboard_state: KeyboardState,

    program_counter: usize,
    cycle_delay: Duration,
    last_cycle: Instant,
}

impl Processor {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = Memory::new();
        memory.load_rom(rom);

        Processor {
            memory,
            registers: Registers::new(),
            stack: Stack::new(),
            display_buf: DisplayBuffer::new(),
            timers: Timers::new(),
            keyboard_state: KeyboardState::new(),

            program_counter: 0x200,
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

        let pc = self.program_counter;
        let opcode = Opcode::new(self.memory.read_byte(pc), self.memory.read_byte(pc + 1));

        let pc_change = OpcodeExecutor::new(opcode).execute(self);

        match pc_change {
            ProgramCounterChange::Next => self.program_counter += 2,
            ProgramCounterChange::Skip => self.program_counter += 4,
            ProgramCounterChange::Jump(loc) => self.program_counter = loc,
            ProgramCounterChange::Wait => (),
        }

        self.timers.tick();
    }

    pub fn get_display_buffer(&self) -> &[bool; DisplayBuffer::SIZE] {
        self.display_buf.buffer()
    }

    pub fn handle_input(&mut self, key_code: VirtualKeyCode, pressed: bool) {
        self.keyboard_state.handle_input(key_code, pressed);
    }
}

/// An opcode for decoding a CHIP-8 instruction.
pub struct Opcode {
    /// Stores the opcode's nibbles, starting at the highest 4 bits.
    ///
    /// Each nibble is guaranteed to be less than 16, i.e. only the lowest 4 bits are used.
    /// `u8` is used as it's the smallest integer primitive for storing 4 bits.
    nibbles: [u8; 4],
}

impl Opcode {
    fn new(upper_byte: u8, lower_byte: u8) -> Self {
        let nibbles = [
            upper_byte >> 4,
            upper_byte & 0x0F,
            lower_byte >> 4,
            lower_byte & 0x0F,
        ];

        Opcode { nibbles }
    }
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for n in &self.nibbles {
            write!(f, "{:X}", n)?;
        }
        Ok(())
    }
}

struct Registers {
    i: u16,
    v: [u8; 16],
}

impl Registers {
    pub fn new() -> Self {
        Registers { i: 0, v: [0; 16] }
    }
}

struct Stack {
    inner: Vec<u16>,
    stack_pointer: u8,
}

impl Stack {
    fn new() -> Self {
        Stack {
            inner: Vec::with_capacity(16),
            stack_pointer: 0,
        }
    }

    fn push(&mut self, value: u16) {
        self.inner.push(value);
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        let val = self.inner.pop().expect("unexpected end of stack");
        self.stack_pointer -= 1;
        val
    }
}
