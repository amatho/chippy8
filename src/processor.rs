use crate::{display::DisplayBuffer, keyboard::KeyboardState, memory::Memory, timer::Timers};
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

        let pc_change = match opcode.uppermost_nibble() {
            0x0 => match opcode.kk() {
                0xE0 => {
                    self.display_buf.clear();
                    ProgramCounterChange::Next
                }
                0xEE => {
                    let pc = self.stack.pop() as usize;
                    ProgramCounterChange::Jump(pc)
                }
                _ => panic!("Invalid opcode {:X}", opcode.0),
            },
            0x1 => ProgramCounterChange::Jump(opcode.nnn() as usize),
            0x2 => {
                // Return from subroutine at next instruction
                self.stack.push((pc + 2) as u16);
                ProgramCounterChange::Jump(opcode.nnn() as usize)
            }
            0x3 => {
                let x = opcode.x() as usize;
                let kk = opcode.kk();
                if self.registers.v[x] == kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            0x4 => {
                let x = opcode.x() as usize;
                let kk = opcode.kk();
                if self.registers.v[x] != kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            0x5 => {
                let x = opcode.x() as usize;
                let y = opcode.y() as usize;
                if self.registers.v[x] == self.registers.v[y] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            0x6 => {
                let x = opcode.x() as usize;
                let kk = opcode.kk();
                self.registers.v[x] = kk;
                ProgramCounterChange::Next
            }
            0x7 => {
                let x = opcode.x() as usize;
                let kk = opcode.kk();
                self.registers.v[x] = self.registers.v[x].wrapping_add(kk);
                ProgramCounterChange::Next
            }
            0x8 => match opcode.n() {
                0x0 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    self.registers.v[x] = self.registers.v[y];
                    ProgramCounterChange::Next
                }
                0x1 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    self.registers.v[x] |= self.registers.v[y];
                    ProgramCounterChange::Next
                }
                0x2 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    self.registers.v[x] &= self.registers.v[y];
                    ProgramCounterChange::Next
                }
                0x3 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    self.registers.v[x] ^= self.registers.v[y];
                    ProgramCounterChange::Next
                }
                0x4 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    let (res, overflowed) =
                        self.registers.v[x].overflowing_add(self.registers.v[y]);
                    self.registers.v[x] = res;
                    self.registers.v[0xF] = overflowed as u8;
                    ProgramCounterChange::Next
                }
                0x5 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    let (res, overflowed) =
                        self.registers.v[x].overflowing_sub(self.registers.v[y]);
                    self.registers.v[x] = res;
                    self.registers.v[0xF] = !overflowed as u8;
                    ProgramCounterChange::Next
                }
                0x6 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    let v_y = self.registers.v[y];
                    let lsb = v_y & 0x1;
                    self.registers.v[x] = v_y >> 1;
                    self.registers.v[0xF] = lsb;
                    ProgramCounterChange::Next
                }
                0x7 => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    let (res, overflowed) =
                        self.registers.v[y].overflowing_sub(self.registers.v[x]);
                    self.registers.v[x] = res;
                    self.registers.v[0xF] = !overflowed as u8;
                    ProgramCounterChange::Next
                }
                0xE => {
                    let x = opcode.x() as usize;
                    let y = opcode.y() as usize;
                    let v_y = self.registers.v[y];
                    let msb = v_y >> 7;
                    self.registers.v[x] = v_y << 1;
                    self.registers.v[0xF] = msb;
                    ProgramCounterChange::Next
                }
                _ => panic!("Invalid opcode {:X}", opcode.0),
            },
            0x9 if opcode.n() == 0 => {
                let x = opcode.x() as usize;
                let y = opcode.y() as usize;
                if self.registers.v[x] != self.registers.v[y] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            0xA => {
                self.registers.i = opcode.nnn();
                ProgramCounterChange::Next
            }
            0xB => {
                let loc = opcode.nnn() + self.registers.v[0x0] as u16;
                ProgramCounterChange::Jump(loc as usize)
            }
            0xC => {
                let x = opcode.x() as usize;
                let kk = opcode.kk();
                let rand_byte = rand::random::<u8>();
                self.registers.v[x] = rand_byte & kk;
                ProgramCounterChange::Next
            }
            0xD => {
                let x = opcode.x() as usize;
                let y = opcode.y() as usize;
                let n = opcode.n() as usize;
                let i = self.registers.i as usize;
                let x_pos = self.registers.v[x] as usize;
                let y_pos = self.registers.v[y] as usize;

                let sprite = self.memory.read_sprite(i, n);
                let collision = self.display_buf.write_sprite(sprite, x_pos, y_pos);
                self.registers.v[0xF] = collision as u8;
                ProgramCounterChange::Next
            }
            0xE => match opcode.kk() {
                0x9E => {
                    let x = opcode.x() as usize;
                    let hex_key = self.registers.v[x] as usize;
                    if self.keyboard_state.key[hex_key] {
                        ProgramCounterChange::Skip
                    } else {
                        ProgramCounterChange::Next
                    }
                }
                0xA1 => {
                    let x = opcode.x() as usize;
                    let hex_key = self.registers.v[x] as usize;
                    if self.keyboard_state.key[hex_key] {
                        ProgramCounterChange::Next
                    } else {
                        ProgramCounterChange::Skip
                    }
                }
                _ => panic!("Invalid opcode {:X}", opcode.0),
            },
            0xF => match opcode.kk() {
                0x07 => {
                    let x = opcode.x() as usize;
                    self.registers.v[x] = self.timers.delay_timer;
                    ProgramCounterChange::Next
                }
                0x0A => {
                    let x = opcode.x() as usize;
                    if let Some(key) = self.keyboard_state.any_pressed() {
                        self.registers.v[x] = key as u8;
                        ProgramCounterChange::Next
                    } else {
                        ProgramCounterChange::Wait
                    }
                }
                0x15 => {
                    let x = opcode.x() as usize;
                    self.timers.delay_timer = self.registers.v[x];
                    ProgramCounterChange::Next
                }
                0x18 => {
                    let x = opcode.x() as usize;
                    self.timers.sound_timer = self.registers.v[x];
                    ProgramCounterChange::Next
                }
                0x1E => {
                    let x = opcode.x() as usize;
                    self.registers.i += self.registers.v[x] as u16;
                    ProgramCounterChange::Next
                }
                0x29 => {
                    let x = opcode.x() as usize;
                    self.registers.i = self.memory.sprite_address(self.registers.v[x]) as u16;
                    ProgramCounterChange::Next
                }
                0x33 => {
                    let x = opcode.x() as usize;
                    let value = self.registers.v[x];
                    let i = self.registers.i as usize;
                    self.memory.write_byte(i, value / 100);
                    self.memory.write_byte(i + 1, value % 100 / 10);
                    self.memory.write_byte(i + 2, value % 10);
                    ProgramCounterChange::Next
                }
                0x55 => {
                    let x = opcode.x() as usize;
                    let i = self.registers.i as usize;
                    for offset in 0..=x {
                        self.memory.write_byte(i + offset, self.registers.v[offset]);
                    }
                    ProgramCounterChange::Next
                }
                0x65 => {
                    let x = opcode.x() as usize;
                    let i = self.registers.i as usize;
                    for offset in 0..=x {
                        self.registers.v[offset] = self.memory.read_byte(i + offset);
                    }
                    ProgramCounterChange::Next
                }
                _ => panic!("Invalid opcode {:X}", opcode.0),
            },
            _ => panic!("Invalid opcode {:X}", opcode.0),
        };

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

enum ProgramCounterChange {
    Next,
    Skip,
    Jump(usize),
    Wait,
}

struct Opcode(u16);

impl Opcode {
    fn new(upper_byte: u8, lower_byte: u8) -> Self {
        let combined = ((upper_byte as u16) << 8) | lower_byte as u16;
        Opcode(combined)
    }

    /// Lowest 12 bits
    fn nnn(&self) -> u16 {
        self.0 & 0x0FFF
    }

    /// Lowest 4 bits
    fn n(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }

    /// Lower 4 bits of the higher byte
    fn x(&self) -> u8 {
        ((self.0 & 0x0F00) >> 8) as u8
    }

    /// Upper 4 bits of the lower byte
    fn y(&self) -> u8 {
        ((self.0 & 0x00F0) >> 4) as u8
    }

    /// Lower byte
    fn kk(&self) -> u8 {
        self.0 as u8
    }

    /// Highest 4 bits
    fn uppermost_nibble(&self) -> u8 {
        (self.0 >> 12) as u8
    }
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
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
        let val = self.inner.pop().expect("Unexpected end of stack");
        self.stack_pointer -= 1;
        val
    }
}
