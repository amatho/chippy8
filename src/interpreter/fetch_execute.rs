use super::{
    instructions::{self as instr, ControlFlow},
    Interpreter,
};
use std::{
    fmt::Debug,
    ops::{BitOr, Shl},
};

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

impl Interpreter {
    pub fn fetch(&mut self) -> Opcode {
        let opcode = Opcode::new(
            self.memory.read_byte(self.program_counter),
            self.memory.read_byte(self.program_counter + 1),
        );

        self.program_counter += 2;

        opcode
    }
}

impl Interpreter {
    pub fn execute(&mut self, opcode: Opcode) {
        let p = self;
        let control_flow = match opcode.nibbles {
            // 00E0
            [0x0, 0x0, 0xE, 0x0] => instr::instr_00E0(p),

            // 00EE
            [0x0, 0x0, 0xE, 0xE] => instr::instr_00EE(p),

            // 1nnn
            [0x1, nnn @ ..] => instr::instr_1nnn(p, combine_nibbles(nnn)),

            // 2nnn
            [0x2, nnn @ ..] => instr::instr_2nnn(p, combine_nibbles(nnn)),

            // 3xkk
            [0x3, x, kk @ ..] => instr::instr_3xkk(p, x, combine_nibbles(kk)),

            // 4xkk
            [0x4, x, kk @ ..] => instr::instr_4xkk(p, x, combine_nibbles(kk)),

            // 5xy0
            [0x5, x, y, 0x0] => instr::instr_5xy0(p, x, y),

            // 6xkk
            [0x6, x, kk @ ..] => instr::instr_6xkk(p, x, combine_nibbles(kk)),

            // 7xkk
            [0x7, x, kk @ ..] => instr::instr_7xkk(p, x, combine_nibbles(kk)),

            // 8xy0
            [0x8, x, y, 0x0] => instr::instr_8xy0(p, x, y),

            // 8xy1
            [0x8, x, y, 0x1] => instr::instr_8xy1(p, x, y),

            // 8xy2
            [0x8, x, y, 0x2] => instr::instr_8xy2(p, x, y),

            // 8xy3
            [0x8, x, y, 0x3] => instr::instr_8xy3(p, x, y),

            // 8xy4
            [0x8, x, y, 0x4] => instr::instr_8xy4(p, x, y),

            // 8xy5
            [0x8, x, y, 0x5] => instr::instr_8xy5(p, x, y),

            // 8xy6
            [0x8, x, y, 0x6] => instr::instr_8xy6(p, x, y),

            // 8xy7
            [0x8, x, y, 0x7] => instr::instr_8xy7(p, x, y),

            // 8xyE
            [0x8, x, y, 0xE] => instr::instr_8xyE(p, x, y),

            // 9xy0
            [0x9, x, y, 0x0] => instr::instr_9xy0(p, x, y),

            // Annn
            [0xA, nnn @ ..] => instr::instr_Annn(p, combine_nibbles(nnn)),

            // Bnnn
            [0xB, nnn @ ..] => instr::instr_Bnnn(p, combine_nibbles(nnn)),

            // Cxkk
            [0xC, x, kk @ ..] => instr::instr_Cxkk(p, x, combine_nibbles(kk)),

            // Dxyn
            [0xD, x, y, n] => instr::instr_Dxyn(p, x, y, n),

            // Ex9E
            [0xE, x, 0x9, 0xE] => instr::instr_Ex9E(p, x),

            // ExA1
            [0xE, x, 0xA, 0x1] => instr::instr_ExA1(p, x),

            // Fx07
            [0xF, x, 0x0, 0x7] => instr::instr_Fx07(p, x),

            // Fx0A
            [0xF, x, 0x0, 0xA] => instr::instr_Fx0A(p, x),

            // Fx15
            [0xF, x, 0x1, 0x5] => instr::instr_Fx15(p, x),

            // Fx18
            [0xF, x, 0x1, 0x8] => instr::instr_Fx18(p, x),

            // Fx1E
            [0xF, x, 0x1, 0xE] => instr::instr_Fx1E(p, x),

            // Fx29
            [0xF, x, 0x2, 0x9] => instr::instr_Fx29(p, x),

            // Fx33
            [0xF, x, 0x3, 0x3] => instr::instr_Fx33(p, x),

            // Fx55
            [0xF, x, 0x5, 0x5] => instr::instr_Fx55(p, x),

            // Fx65
            [0xF, x, 0x6, 0x5] => instr::instr_Fx65(p, x),

            _ => panic!("invalid opcode: {:?}", opcode),
        };

        match control_flow {
            ControlFlow::Wait => p.program_counter -= 2,
            ControlFlow::Skip => p.program_counter += 2,
            ControlFlow::Jump(loc) => p.program_counter = loc as usize,
            ControlFlow::None => (),
        }
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
