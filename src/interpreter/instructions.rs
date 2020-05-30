use super::Interpreter;

pub struct InstructionExecutor<'a> {
    interp: &'a mut Interpreter,
}

#[allow(non_snake_case)]
impl<'a> InstructionExecutor<'a> {
    pub fn new(p: &'a mut Interpreter) -> Self {
        Self { interp: p }
    }

    pub fn close(self) -> &'a mut Interpreter {
        self.interp
    }

    pub fn instr_00E0(&mut self) -> ProgramCounterChange {
        self.interp.display_buf.clear();
        ProgramCounterChange::Next
    }

    pub fn instr_00EE(&mut self) -> ProgramCounterChange {
        let pc = self.interp.stack.pop() as usize;
        ProgramCounterChange::Jump(pc)
    }

    pub fn instr_1nnn(&mut self, nnn: usize) -> ProgramCounterChange {
        ProgramCounterChange::Jump(nnn)
    }

    pub fn instr_2nnn(&mut self, nnn: usize) -> ProgramCounterChange {
        // Return from subroutine at next instruction
        self.interp
            .stack
            .push((self.interp.program_counter + 2) as u16);
        ProgramCounterChange::Jump(nnn)
    }

    pub fn instr_3xkk(&mut self, x: u8, kk: u8) -> ProgramCounterChange {
        if self.interp.reg_v[x as usize] == kk {
            ProgramCounterChange::Skip
        } else {
            ProgramCounterChange::Next
        }
    }

    pub fn instr_4xkk(&mut self, x: u8, kk: u8) -> ProgramCounterChange {
        if self.interp.reg_v[x as usize] != kk {
            ProgramCounterChange::Skip
        } else {
            ProgramCounterChange::Next
        }
    }

    pub fn instr_5xy0(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        if self.interp.reg_v[x as usize] == self.interp.reg_v[y as usize] {
            ProgramCounterChange::Skip
        } else {
            ProgramCounterChange::Next
        }
    }

    pub fn instr_6xkk(&mut self, x: u8, kk: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] = kk;
        ProgramCounterChange::Next
    }

    pub fn instr_7xkk(&mut self, x: u8, kk: u8) -> ProgramCounterChange {
        let x = x as usize;
        self.interp.reg_v[x] = self.interp.reg_v[x].wrapping_add(kk);
        ProgramCounterChange::Next
    }

    pub fn instr_8xy0(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] = self.interp.reg_v[y as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_8xy1(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] |= self.interp.reg_v[y as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_8xy2(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] &= self.interp.reg_v[y as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_8xy3(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] ^= self.interp.reg_v[y as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_8xy4(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        let v_y = self.interp.reg_v[y as usize];
        let v_x = &mut self.interp.reg_v[x as usize];
        let (sum, overflowed) = v_x.overflowing_add(v_y);
        *v_x = sum;
        self.interp.reg_v[0xF] = overflowed as u8;
        ProgramCounterChange::Next
    }

    pub fn instr_8xy5(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        let v_y = self.interp.reg_v[y as usize];
        let v_x = &mut self.interp.reg_v[x as usize];
        let (diff, overflowed) = v_x.overflowing_sub(v_y);
        *v_x = diff;
        self.interp.reg_v[0xF] = !overflowed as u8;
        ProgramCounterChange::Next
    }

    pub fn instr_8xy6(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        let x = x as usize;
        let y = y as usize;
        let v_y = self.interp.reg_v[y];
        let lsb = v_y & 0x1;
        self.interp.reg_v[x] = v_y >> 1;
        self.interp.reg_v[0xF] = lsb;
        ProgramCounterChange::Next
    }

    pub fn instr_8xy7(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        let v_y = self.interp.reg_v[y as usize];
        let v_x = &mut self.interp.reg_v[x as usize];
        let (diff, overflowed) = v_y.overflowing_sub(*v_x);
        *v_x = diff;
        self.interp.reg_v[0xF] = !overflowed as u8;
        ProgramCounterChange::Next
    }

    pub fn instr_8xyE(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        let v_y = self.interp.reg_v[y as usize];
        let msb = v_y >> 7;
        self.interp.reg_v[x as usize] = v_y << 1;
        self.interp.reg_v[0xF] = msb;
        ProgramCounterChange::Next
    }

    pub fn instr_9xy0(&mut self, x: u8, y: u8) -> ProgramCounterChange {
        if self.interp.reg_v[x as usize] != self.interp.reg_v[y as usize] {
            ProgramCounterChange::Skip
        } else {
            ProgramCounterChange::Next
        }
    }

    pub fn instr_Annn(&mut self, nnn: u16) -> ProgramCounterChange {
        self.interp.reg_i = nnn;
        ProgramCounterChange::Next
    }

    pub fn instr_Bnnn(&mut self, nnn: u16) -> ProgramCounterChange {
        let loc = nnn + self.interp.reg_v[0x0] as u16;
        ProgramCounterChange::Jump(loc as usize)
    }

    pub fn instr_Cxkk(&mut self, x: u8, kk: u8) -> ProgramCounterChange {
        let rand_byte = rand::random::<u8>();
        self.interp.reg_v[x as usize] = rand_byte & kk;
        ProgramCounterChange::Next
    }

    pub fn instr_Dxyn(&mut self, x: u8, y: u8, n: u8) -> ProgramCounterChange {
        let p = &mut self.interp;
        let x_pos = p.reg_v[x as usize] as usize;
        let y_pos = p.reg_v[y as usize] as usize;

        if x_pos > 0x3F || y_pos > 0x1F {
            p.reg_v[0xF] = 0;
            return ProgramCounterChange::Next;
        }

        let sprite = p.memory.read_sprite(p.reg_i as usize, n as usize);
        let collision = p.display_buf.write_sprite(sprite, x_pos, y_pos);
        p.reg_v[0xF] = collision as u8;
        ProgramCounterChange::Next
    }

    pub fn instr_Ex9E(&mut self, x: u8) -> ProgramCounterChange {
        let hex_key = self.interp.reg_v[x as usize] as usize;
        if self.interp.keyboard_state.key[hex_key] {
            ProgramCounterChange::Skip
        } else {
            ProgramCounterChange::Next
        }
    }

    pub fn instr_ExA1(&mut self, x: u8) -> ProgramCounterChange {
        let hex_key = self.interp.reg_v[x as usize] as usize;
        if self.interp.keyboard_state.key[hex_key] {
            ProgramCounterChange::Next
        } else {
            ProgramCounterChange::Skip
        }
    }

    pub fn instr_Fx07(&mut self, x: u8) -> ProgramCounterChange {
        self.interp.reg_v[x as usize] = self.interp.timers.delay_timer;
        ProgramCounterChange::Next
    }

    pub fn instr_Fx0A(&mut self, x: u8) -> ProgramCounterChange {
        if let Some(key) = self.interp.keyboard_state.any_pressed() {
            self.interp.reg_v[x as usize] = key as u8;
            ProgramCounterChange::Next
        } else {
            ProgramCounterChange::Wait
        }
    }

    pub fn instr_Fx15(&mut self, x: u8) -> ProgramCounterChange {
        self.interp.timers.delay_timer = self.interp.reg_v[x as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_Fx18(&mut self, x: u8) -> ProgramCounterChange {
        self.interp.timers.sound_timer = self.interp.reg_v[x as usize];
        ProgramCounterChange::Next
    }

    pub fn instr_Fx1E(&mut self, x: u8) -> ProgramCounterChange {
        self.interp.reg_i += self.interp.reg_v[x as usize] as u16;
        ProgramCounterChange::Next
    }

    pub fn instr_Fx29(&mut self, x: u8) -> ProgramCounterChange {
        self.interp.reg_i = self
            .interp
            .memory
            .sprite_address(self.interp.reg_v[x as usize]) as u16;
        ProgramCounterChange::Next
    }

    pub fn instr_Fx33(&mut self, x: u8) -> ProgramCounterChange {
        let value = self.interp.reg_v[x as usize];
        let i = self.interp.reg_i as usize;
        self.interp.memory.write_byte(i, value / 100);
        self.interp.memory.write_byte(i + 1, value % 100 / 10);
        self.interp.memory.write_byte(i + 2, value % 10);
        ProgramCounterChange::Next
    }

    pub fn instr_Fx55(&mut self, x: u8) -> ProgramCounterChange {
        let x = x as usize;
        let i = self.interp.reg_i as usize;
        for offset in 0..=x {
            self.interp
                .memory
                .write_byte(i + offset, self.interp.reg_v[offset]);
        }
        ProgramCounterChange::Next
    }

    pub fn instr_Fx65(&mut self, x: u8) -> ProgramCounterChange {
        let x = x as usize;
        let i = self.interp.reg_i as usize;
        for offset in 0..=x {
            self.interp.reg_v[offset] = self.interp.memory.read_byte(i + offset);
        }
        ProgramCounterChange::Next
    }
}

pub enum ProgramCounterChange {
    Next,
    Skip,
    Jump(usize),
    Wait,
}
