use super::{Opcode, Processor};
use std::ops::{BitOr, Shl};

pub struct OpcodeExecutor(Opcode);

impl OpcodeExecutor {
    pub fn new(opcode: Opcode) -> Self {
        OpcodeExecutor(opcode)
    }

    pub fn execute(&self, processor: &mut Processor) -> ProgramCounterChange {
        let p = processor;
        let pc = p.program_counter;

        match self.0.nibbles {
            // 00E0
            [0x0, 0x0, 0xE, 0x0] => {
                p.display_buf.clear();
                ProgramCounterChange::Next
            }
            // 00EE
            [0x0, 0x0, 0xE, 0xE] => {
                let pc = p.stack.pop() as usize;
                ProgramCounterChange::Jump(pc)
            }
            // 1nnn
            [0x1, nnn @ ..] => ProgramCounterChange::Jump(combine_nibbles(nnn)),
            // 2nnn
            [0x2, nnn @ ..] => {
                // Return from subroutine at next instruction
                p.stack.push((pc + 2) as u16);
                ProgramCounterChange::Jump(combine_nibbles(nnn))
            }
            // 3xkk
            [0x3, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                if p.registers.v[x as usize] == kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            // 4xkk
            [0x4, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                if p.registers.v[x as usize] != kk {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            // 5xy0
            [0x5, x, y, 0x0] => {
                if p.registers.v[x as usize] == p.registers.v[y as usize] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            // 6xkk
            [0x6, x, kk @ ..] => {
                let kk = combine_nibbles(kk);
                p.registers.v[x as usize] = kk;
                ProgramCounterChange::Next
            }
            // 7xkk
            [0x7, x, kk @ ..] => {
                let x = x as usize;
                let kk = combine_nibbles(kk);
                p.registers.v[x] = p.registers.v[x].wrapping_add(kk);
                ProgramCounterChange::Next
            }
            // 8xy0
            [0x8, x, y, 0x0] => {
                p.registers.v[x as usize] = p.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            // 8xy1
            [0x8, x, y, 0x1] => {
                p.registers.v[x as usize] |= p.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            // 8xy2
            [0x8, x, y, 0x2] => {
                p.registers.v[x as usize] &= p.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            // 8xy3
            [0x8, x, y, 0x3] => {
                p.registers.v[x as usize] ^= p.registers.v[y as usize];
                ProgramCounterChange::Next
            }
            // 8xy4
            [0x8, x, y, 0x4] => {
                let v_y = p.registers.v[y as usize];
                let v_x = &mut p.registers.v[x as usize];
                let (sum, overflowed) = v_x.overflowing_add(v_y);
                *v_x = sum;
                p.registers.v[0xF] = overflowed as u8;
                ProgramCounterChange::Next
            }
            // 8xy5
            [0x8, x, y, 0x5] => {
                let v_y = p.registers.v[y as usize];
                let v_x = &mut p.registers.v[x as usize];
                let (diff, overflowed) = v_x.overflowing_sub(v_y);
                *v_x = diff;
                p.registers.v[0xF] = !overflowed as u8;
                ProgramCounterChange::Next
            }
            // 8xy6
            [0x8, x, y, 0x6] => {
                let x = x as usize;
                let y = y as usize;
                let v_y = p.registers.v[y];
                let lsb = v_y & 0x1;
                p.registers.v[x] = v_y >> 1;
                p.registers.v[0xF] = lsb;
                ProgramCounterChange::Next
            }
            // 8xy7
            [0x8, x, y, 0x7] => {
                let v_y = p.registers.v[y as usize];
                let v_x = &mut p.registers.v[x as usize];
                let (diff, overflowed) = v_y.overflowing_sub(*v_x);
                *v_x = diff;
                p.registers.v[0xF] = !overflowed as u8;
                ProgramCounterChange::Next
            }
            // 8xyE
            [0x8, x, y, 0xE] => {
                let v_y = p.registers.v[y as usize];
                let msb = v_y >> 7;
                p.registers.v[x as usize] = v_y << 1;
                p.registers.v[0xF] = msb;
                ProgramCounterChange::Next
            }
            // 9xy0
            [0x9, x, y, 0x0] => {
                if p.registers.v[x as usize] != p.registers.v[y as usize] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            // Annn
            [0xA, nnn @ ..] => {
                p.registers.i = combine_nibbles(nnn);
                ProgramCounterChange::Next
            }
            // Bnnn
            [0xB, nnn @ ..] => {
                let nnn: u16 = combine_nibbles(nnn);
                let loc = nnn + p.registers.v[0x0] as u16;
                ProgramCounterChange::Jump(loc as usize)
            }
            // Cxkk
            [0xC, x, kk @ ..] => {
                let kk: u8 = combine_nibbles(kk);
                let rand_byte = rand::random::<u8>();
                p.registers.v[x as usize] = rand_byte & kk;
                ProgramCounterChange::Next
            }
            // Dxyn
            [0xD, x, y, n] => {
                let x_pos = p.registers.v[x as usize] as usize;
                let y_pos = p.registers.v[y as usize] as usize;

                let sprite = p.memory.read_sprite(p.registers.i as usize, n as usize);
                let collision = p.display_buf.write_sprite(sprite, x_pos, y_pos);
                p.registers.v[0xF] = collision as u8;
                ProgramCounterChange::Next
            }
            // Ex9E
            [0xE, x, 0x9, 0xE] => {
                let hex_key = p.registers.v[x as usize] as usize;
                if p.keyboard_state.key[hex_key] {
                    ProgramCounterChange::Skip
                } else {
                    ProgramCounterChange::Next
                }
            }
            // ExA1
            [0xE, x, 0xA, 0x1] => {
                let hex_key = p.registers.v[x as usize] as usize;
                if p.keyboard_state.key[hex_key] {
                    ProgramCounterChange::Next
                } else {
                    ProgramCounterChange::Skip
                }
            }
            // Fx07
            [0xF, x, 0x0, 0x7] => {
                p.registers.v[x as usize] = p.timers.delay_timer;
                ProgramCounterChange::Next
            }
            // Fx0A
            [0xF, x, 0x0, 0xA] => {
                if let Some(key) = p.keyboard_state.any_pressed() {
                    p.registers.v[x as usize] = key as u8;
                    ProgramCounterChange::Next
                } else {
                    ProgramCounterChange::Wait
                }
            }
            // Fx15
            [0xF, x, 0x1, 0x5] => {
                p.timers.delay_timer = p.registers.v[x as usize];
                ProgramCounterChange::Next
            }
            // Fx18
            [0xF, x, 0x1, 0x8] => {
                p.timers.sound_timer = p.registers.v[x as usize];
                ProgramCounterChange::Next
            }
            // Fx1E
            [0xF, x, 0x1, 0xE] => {
                p.registers.i += p.registers.v[x as usize] as u16;
                ProgramCounterChange::Next
            }
            // Fx29
            [0xF, x, 0x2, 0x9] => {
                p.registers.i = p.memory.sprite_address(p.registers.v[x as usize]) as u16;
                ProgramCounterChange::Next
            }
            // Fx33
            [0xF, x, 0x3, 0x3] => {
                let value = p.registers.v[x as usize];
                let i = p.registers.i as usize;
                p.memory.write_byte(i, value / 100);
                p.memory.write_byte(i + 1, value % 100 / 10);
                p.memory.write_byte(i + 2, value % 10);
                ProgramCounterChange::Next
            }
            // Fx55
            [0xF, x, 0x5, 0x5] => {
                let x = x as usize;
                let i = p.registers.i as usize;
                for offset in 0..=x {
                    p.memory.write_byte(i + offset, p.registers.v[offset]);
                }
                ProgramCounterChange::Next
            }
            // Fx65
            [0xF, x, 0x6, 0x5] => {
                let x = x as usize;
                let i = p.registers.i as usize;
                for offset in 0..=x {
                    p.registers.v[offset] = p.memory.read_byte(i + offset);
                }
                ProgramCounterChange::Next
            }
            _ => panic!("invalid opcode: {:?}", self.0),
        }
    }
}

pub enum ProgramCounterChange {
    Next,
    Skip,
    Jump(usize),
    Wait,
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
