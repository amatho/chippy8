#![allow(non_snake_case)]

use super::Interpreter;

pub enum ControlFlow {
    Wait,
    Skip,
    Jump(u16),
    None,
}

pub fn instr_00E0(interp: &mut Interpreter) -> ControlFlow {
    interp.display_buf.clear();
    ControlFlow::None
}

pub fn instr_00EE(interp: &mut Interpreter) -> ControlFlow {
    let pc = interp.stack.pop().unwrap();
    ControlFlow::Jump(pc)
}

pub fn instr_1nnn(_interp: &mut Interpreter, nnn: u16) -> ControlFlow {
    ControlFlow::Jump(nnn)
}

pub fn instr_2nnn(interp: &mut Interpreter, nnn: u16) -> ControlFlow {
    // Return from subroutine at next instruction
    interp.stack.push((interp.program_counter) as u16);
    ControlFlow::Jump(nnn)
}

pub fn instr_3xkk(interp: &mut Interpreter, x: u8, kk: u8) -> ControlFlow {
    if interp.reg_v(x) == kk {
        ControlFlow::Skip
    } else {
        ControlFlow::None
    }
}

pub fn instr_4xkk(interp: &mut Interpreter, x: u8, kk: u8) -> ControlFlow {
    if interp.reg_v(x) != kk {
        ControlFlow::Skip
    } else {
        ControlFlow::None
    }
}

pub fn instr_5xy0(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    if interp.reg_v(x) == interp.reg_v(y) {
        ControlFlow::Skip
    } else {
        ControlFlow::None
    }
}

pub fn instr_6xkk(interp: &mut Interpreter, x: u8, kk: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = kk;
    ControlFlow::None
}

pub fn instr_7xkk(interp: &mut Interpreter, x: u8, kk: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.reg_v(x).wrapping_add(kk);
    ControlFlow::None
}

pub fn instr_8xy0(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.reg_v(y);
    ControlFlow::None
}

pub fn instr_8xy1(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.reg_v(x) | interp.reg_v(y);
    ControlFlow::None
}

pub fn instr_8xy2(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.reg_v(x) & interp.reg_v(y);
    ControlFlow::None
}

pub fn instr_8xy3(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.reg_v(x) ^ interp.reg_v(y);
    ControlFlow::None
}

pub fn instr_8xy4(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    let v_y = interp.reg_v(y);
    let v_x = interp.reg_v_mut(x);
    let (sum, overflowed) = v_x.overflowing_add(v_y);
    *v_x = sum;
    interp.reg_v[0xF] = overflowed as u8;
    // println!("{:?}", (interp.reg_v(x), interp.reg_v(y)));
    ControlFlow::None
}

pub fn instr_8xy5(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    let v_y = interp.reg_v(y);
    let v_x = interp.reg_v_mut(x);
    let (diff, overflowed) = v_x.overflowing_sub(v_y);
    *v_x = diff;
    interp.reg_v[0xF] = !overflowed as u8;
    ControlFlow::None
}

pub fn instr_8xy6(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    let x = x as usize;
    let y = y as usize;
    let v_y = interp.reg_v[y];
    let lsb = v_y & 0x1;
    interp.reg_v[x] = v_y >> 1;
    interp.reg_v[0xF] = lsb;
    ControlFlow::None
}

pub fn instr_8xy7(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    let v_y = interp.reg_v(y);
    let v_x = interp.reg_v_mut(x);
    let (diff, overflowed) = v_y.overflowing_sub(*v_x);
    *v_x = diff;
    interp.reg_v[0xF] = !overflowed as u8;
    ControlFlow::None
}

pub fn instr_8xyE(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    let v_y = interp.reg_v(y);
    let msb = v_y >> 7;
    *interp.reg_v_mut(x) = v_y << 1;
    interp.reg_v[0xF] = msb;
    ControlFlow::None
}

pub fn instr_9xy0(interp: &mut Interpreter, x: u8, y: u8) -> ControlFlow {
    if interp.reg_v(x) != interp.reg_v(y) {
        ControlFlow::Skip
    } else {
        ControlFlow::None
    }
}

pub fn instr_Annn(interp: &mut Interpreter, nnn: u16) -> ControlFlow {
    interp.reg_i = nnn;
    ControlFlow::None
}

pub fn instr_Bnnn(interp: &mut Interpreter, nnn: u16) -> ControlFlow {
    let loc = nnn + interp.reg_v[0x0] as u16;
    ControlFlow::Jump(loc)
}

pub fn instr_Cxkk(interp: &mut Interpreter, x: u8, kk: u8) -> ControlFlow {
    let rand_byte = rand::random::<u8>();
    *interp.reg_v_mut(x) = rand_byte & kk;
    ControlFlow::None
}

pub fn instr_Dxyn(interp: &mut Interpreter, x: u8, y: u8, n: u8) -> ControlFlow {
    let p = interp;
    let x_pos = p.reg_v(x) as usize;
    let y_pos = p.reg_v(y) as usize;

    if x_pos > 0x3F || y_pos > 0x1F {
        p.reg_v[0xF] = 0;
        return ControlFlow::None;
    }

    let sprite = p.memory.read_sprite(p.reg_i as usize, n as usize);
    let collision = p.display_buf.write_sprite(sprite, x_pos, y_pos);
    p.reg_v[0xF] = collision as u8;
    ControlFlow::None
}

pub fn instr_Ex9E(interp: &mut Interpreter, x: u8) -> ControlFlow {
    let hex_key = interp.reg_v(x) as usize;
    if interp.keyboard_state.key[hex_key] {
        ControlFlow::Skip
    } else {
        ControlFlow::None
    }
}

pub fn instr_ExA1(interp: &mut Interpreter, x: u8) -> ControlFlow {
    let hex_key = interp.reg_v(x) as usize;
    if interp.keyboard_state.key[hex_key] {
        ControlFlow::None
    } else {
        ControlFlow::Skip
    }
}

pub fn instr_Fx07(interp: &mut Interpreter, x: u8) -> ControlFlow {
    *interp.reg_v_mut(x) = interp.timers.delay_timer;
    ControlFlow::None
}

pub fn instr_Fx0A(interp: &mut Interpreter, x: u8) -> ControlFlow {
    if let Some(key) = interp.keyboard_state.any_pressed() {
        *interp.reg_v_mut(x) = key as u8;
        ControlFlow::None
    } else {
        ControlFlow::Wait
    }
}

pub fn instr_Fx15(interp: &mut Interpreter, x: u8) -> ControlFlow {
    interp.timers.delay_timer = interp.reg_v(x);
    ControlFlow::None
}

pub fn instr_Fx18(interp: &mut Interpreter, x: u8) -> ControlFlow {
    interp.timers.sound_timer = interp.reg_v(x);
    ControlFlow::None
}

pub fn instr_Fx1E(interp: &mut Interpreter, x: u8) -> ControlFlow {
    interp.reg_i += interp.reg_v(x) as u16;
    ControlFlow::None
}

pub fn instr_Fx29(interp: &mut Interpreter, x: u8) -> ControlFlow {
    interp.reg_i = interp.memory.sprite_address(interp.reg_v(x)) as u16;
    ControlFlow::None
}

pub fn instr_Fx33(interp: &mut Interpreter, x: u8) -> ControlFlow {
    let value = interp.reg_v(x);
    let i = interp.reg_i as usize;
    interp.memory.write_byte(i, value / 100);
    interp.memory.write_byte(i + 1, value % 100 / 10);
    interp.memory.write_byte(i + 2, value % 10);
    ControlFlow::None
}

pub fn instr_Fx55(interp: &mut Interpreter, x: u8) -> ControlFlow {
    let x = x as usize;
    let i = interp.reg_i as usize;
    for offset in 0..=x {
        interp.memory.write_byte(i + offset, interp.reg_v[offset]);
    }
    ControlFlow::None
}

pub fn instr_Fx65(interp: &mut Interpreter, x: u8) -> ControlFlow {
    let x = x as usize;
    let i = interp.reg_i as usize;
    for offset in 0..=x {
        interp.reg_v[offset] = interp.memory.read_byte(i + offset);
    }
    ControlFlow::None
}
