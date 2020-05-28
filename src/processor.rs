use crate::{display::DisplayBuffer, keyboard::KeyboardState, memory::Memory, timer::Timers};
use std::{
    fmt::Debug,
    ops::{BitOr, Shl},
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

        let pc_change = match opcode.nibbles {
            [0x0, 0x0, 0xE, 0x0] => {
                self.display_buf.clear();
                ProgramCounterChange::Next
            }
            [0x0, 0x0, 0xE, 0xE] => {
                let pc = self.stack.pop() as usize;
                ProgramCounterChange::Jump(pc)
            }
            [0x1, nnn @ ..] => ProgramCounterChange::Jump(combine_nibbles(nnn)),
            [0x2, nnn @ ..] => {
                // Return from subroutine at next instruction
                self.stack.push((pc + 2) as u16);
                ProgramCounterChange::Jump(combine_nibbles(nnn))
            }
            [0x3, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                if self.registers.v[x as usize] == kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            [0x4, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                if self.registers.v[x as usize] != kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            [0x5, x, y, 0x0] => {
                if self.registers.v[x as usize] == self.registers.v[y as usize] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            [0x6, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                self.registers.v[x as usize] = kk;
                ProgramCounterChange::Next
            }
            [0x7, x, kk @ ..] => {
                let x = x as usize;
                let kk = combine_nibbles(kk);
                self.registers.v[x] = self.registers.v[x].wrapping_add(kk);
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x0] => {
                self.registers.v[x as usize] = self.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x1] => {
                self.registers.v[x as usize] |= self.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x2] => {
                self.registers.v[x as usize] &= self.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x3] => {
                self.registers.v[x as usize] ^= self.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x4] => {
                let v_y = self.registers.v[y as usize];
                let v_x = &mut self.registers.v[x as usize];
                let (sum, overflowed) = v_x.overflowing_add(v_y);
                *v_x = sum;
                self.registers.v[0xF] = overflowed as u8;
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x5] => {
                let v_y = self.registers.v[y as usize];
                let v_x = &mut self.registers.v[x as usize];
                let (diff, overflowed) = v_x.overflowing_sub(v_y);
                *v_x = diff;
                self.registers.v[0xF] = !overflowed as u8;
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x6] => {
                let x = x as usize;
                let y = y as usize;
                let v_y = self.registers.v[y];
                let lsb = v_y & 0x1;
                self.registers.v[x] = v_y >> 1;
                self.registers.v[0xF] = lsb;
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0x7] => {
                let v_y = self.registers.v[y as usize];
                let v_x = &mut self.registers.v[x as usize];
                let (diff, overflowed) = v_y.overflowing_sub(*v_x);
                *v_x = diff;
                self.registers.v[0xF] = !overflowed as u8;
                ProgramCounterChange::Next
            }
            [0x8, x, y, 0xE] => {
                let v_y = self.registers.v[y as usize];
                let msb = v_y >> 7;
                self.registers.v[x as usize] = v_y << 1;
                self.registers.v[0xF] = msb;
                ProgramCounterChange::Next
            }
            [0x9, x, y, 0x0] => {
                if self.registers.v[x as usize] != self.registers.v[y as usize] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            [0xA, nnn @ ..] => {
                self.registers.i = combine_nibbles(nnn);
                ProgramCounterChange::Next
            }
            [0xB, nnn @ ..] => {
                let nnn: u16 = combine_nibbles(nnn);
                let loc = nnn + self.registers.v[0x0] as u16;
                ProgramCounterChange::Jump(loc as usize)
            }
            [0xC, x, kk @ ..] => {
                let kk: u8 = combine_nibbles(kk);
                let rand_byte = rand::random::<u8>();
                self.registers.v[x as usize] = rand_byte & kk;
                ProgramCounterChange::Next
            }
            [0xD, x, y, n] => {
                let x_pos = self.registers.v[x as usize] as usize;
                let y_pos = self.registers.v[y as usize] as usize;

                let sprite = self
                    .memory
                    .read_sprite(self.registers.i as usize, n as usize);
                let collision = self.display_buf.write_sprite(sprite, x_pos, y_pos);
                self.registers.v[0xF] = collision as u8;
                ProgramCounterChange::Next
            }
            [0xE, x, 0x9, 0xE] => {
                let hex_key = self.registers.v[x as usize] as usize;
                if self.keyboard_state.key[hex_key] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            [0xE, x, 0xA, 0x1] => {
                let hex_key = self.registers.v[x as usize] as usize;
                if self.keyboard_state.key[hex_key] {
                    ProgramCounterChange::Next
                } else {
                    ProgramCounterChange::Skip
                }
            }
            [0xF, x, 0x0, 0x7] => {
                self.registers.v[x as usize] = self.timers.delay_timer;
                ProgramCounterChange::Next
            }
            [0xF, x, 0x0, 0xA] => {
                if let Some(key) = self.keyboard_state.any_pressed() {
                    self.registers.v[x as usize] = key as u8;
                    ProgramCounterChange::Next
                } else {
                    ProgramCounterChange::Wait
                }
            }
            [0xF, x, 0x1, 0x5] => {
                self.timers.delay_timer = self.registers.v[x as usize];
                ProgramCounterChange::Next
            }
            [0xF, x, 0x1, 0x8] => {
                self.timers.sound_timer = self.registers.v[x as usize];
                ProgramCounterChange::Next
            }
            [0xF, x, 0x1, 0xE] => {
                self.registers.i += self.registers.v[x as usize] as u16;
                ProgramCounterChange::Next
            }
            [0xF, x, 0x2, 0x9] => {
                self.registers.i = self.memory.sprite_address(self.registers.v[x as usize]) as u16;
                ProgramCounterChange::Next
            }
            [0xF, x, 0x3, 0x3] => {
                let value = self.registers.v[x as usize];
                let i = self.registers.i as usize;
                self.memory.write_byte(i, value / 100);
                self.memory.write_byte(i + 1, value % 100 / 10);
                self.memory.write_byte(i + 2, value % 10);
                ProgramCounterChange::Next
            }
            [0xF, x, 0x5, 0x5] => {
                let x = x as usize;
                let i = self.registers.i as usize;
                for offset in 0..=x {
                    self.memory.write_byte(i + offset, self.registers.v[offset]);
                }
                ProgramCounterChange::Next
            }
            [0xF, x, 0x6, 0x5] => {
                let x = x as usize;
                let i = self.registers.i as usize;
                for offset in 0..=x {
                    self.registers.v[offset] = self.memory.read_byte(i + offset);
                }
                ProgramCounterChange::Next
            }
            _ => panic!("invalid opcode: {:?}", opcode),
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

struct Opcode {
    /// Stores the opcode's nibbles, starting at the highest 4 bits
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

fn combine_nibbles<T, U>(nibbles: T) -> U
where
    T: AsRef<[u8]>,
    U: From<u8> + Shl<Output = U> + BitOr<Output = U>,
{
    let nibbles = nibbles.as_ref();
    assert!((nibbles.len() / 2) <= std::mem::size_of::<T>());
    let mut iter = nibbles.iter();
    let mut result = U::from(*iter.next().expect("nibbles must have at least 1 element"));
    for &nibble in iter {
        result = result << U::from(4) | U::from(nibble);
    }
    result
}
