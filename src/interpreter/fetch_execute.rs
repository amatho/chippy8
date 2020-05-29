use super::{
    instructions::{InstructionExecutor, ProgramCounterChange},
    Interpreter,
};
use std::{
    fmt::Debug,
    ops::{BitOr, Shl},
};

/// An opcode for decoding a CHIP-8 instruction.
struct Opcode {
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

pub struct OpcodeFetchExecute<'a> {
    opcode: Opcode,
    executor: InstructionExecutor<'a>,
}

impl<'a> OpcodeFetchExecute<'a> {
    pub fn fetch(p: &'a mut Interpreter) -> Self {
        let opcode = Opcode::new(
            p.memory.read_byte(p.program_counter),
            p.memory.read_byte(p.program_counter + 1),
        );
        let executor = InstructionExecutor::new(p);

        Self { opcode, executor }
    }

    pub fn execute(mut self) {
        let pc_change = match self.opcode.nibbles {
            // 00E0
            [0x0, 0x0, 0xE, 0x0] => self.executor.instr_00E0(),

            // 00EE
            [0x0, 0x0, 0xE, 0xE] => self.executor.instr_00EE(),

            // 1nnn
            [0x1, nnn @ ..] => self.executor.instr_1nnn(combine_nibbles(nnn)),

            // 2nnn
            [0x2, nnn @ ..] => self.executor.instr_2nnn(combine_nibbles(nnn)),

            // 3xkk
            [0x3, x, kk @ ..] => self.executor.instr_3xkk(x, combine_nibbles(kk)),

            // 4xkk
            [0x4, x, kk @ ..] => self.executor.instr_4xkk(x, combine_nibbles(kk)),

            // 5xy0
            [0x5, x, y, 0x0] => self.executor.instr_5xy0(x, y),

            // 6xkk
            [0x6, x, kk @ ..] => self.executor.instr_6xkk(x, combine_nibbles(kk)),

            // 7xkk
            [0x7, x, kk @ ..] => self.executor.instr_7xkk(x, combine_nibbles(kk)),

            // 8xy0
            [0x8, x, y, 0x0] => self.executor.instr_8xy0(x, y),

            // 8xy1
            [0x8, x, y, 0x1] => self.executor.instr_8xy1(x, y),

            // 8xy2
            [0x8, x, y, 0x2] => self.executor.instr_8xy2(x, y),

            // 8xy3
            [0x8, x, y, 0x3] => self.executor.instr_8xy3(x, y),

            // 8xy4
            [0x8, x, y, 0x4] => self.executor.instr_8xy4(x, y),

            // 8xy5
            [0x8, x, y, 0x5] => self.executor.instr_8xy5(x, y),

            // 8xy6
            [0x8, x, y, 0x6] => self.executor.instr_8xy6(x, y),

            // 8xy7
            [0x8, x, y, 0x7] => self.executor.instr_8xy7(x, y),

            // 8xyE
            [0x8, x, y, 0xE] => self.executor.instr_8xyE(x, y),

            // 9xy0
            [0x9, x, y, 0x0] => self.executor.instr_9xy0(x, y),

            // Annn
            [0xA, nnn @ ..] => self.executor.instr_Annn(combine_nibbles(nnn)),

            // Bnnn
            [0xB, nnn @ ..] => self.executor.instr_Bnnn(combine_nibbles(nnn)),

            // Cxkk
            [0xC, x, kk @ ..] => self.executor.instr_Cxkk(x, combine_nibbles(kk)),

            // Dxyn
            [0xD, x, y, n] => self.executor.instr_Dxyn(x, y, n),

            // Ex9E
            [0xE, x, 0x9, 0xE] => self.executor.instr_Ex9E(x),

            // ExA1
            [0xE, x, 0xA, 0x1] => self.executor.instr_ExA1(x),

            // Fx07
            [0xF, x, 0x0, 0x7] => self.executor.instr_Fx07(x),

            // Fx0A
            [0xF, x, 0x0, 0xA] => self.executor.instr_Fx0A(x),

            // Fx15
            [0xF, x, 0x1, 0x5] => self.executor.instr_Fx15(x),

            // Fx18
            [0xF, x, 0x1, 0x8] => self.executor.instr_Fx18(x),

            // Fx1E
            [0xF, x, 0x1, 0xE] => self.executor.instr_Fx1E(x),

            // Fx29
            [0xF, x, 0x2, 0x9] => self.executor.instr_Fx29(x),

            // Fx33
            [0xF, x, 0x3, 0x3] => self.executor.instr_Fx33(x),

            // Fx55
            [0xF, x, 0x5, 0x5] => self.executor.instr_Fx55(x),

            // Fx65
            [0xF, x, 0x6, 0x5] => self.executor.instr_Fx65(x),

            _ => panic!("invalid opcode: {:?}", self.opcode),
        };

        let p = self.executor.close();
        match pc_change {
            ProgramCounterChange::Next => p.program_counter += 2,
            ProgramCounterChange::Skip => p.program_counter += 4,
            ProgramCounterChange::Jump(loc) => p.program_counter = loc,
            ProgramCounterChange::Wait => (),
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
