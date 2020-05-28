use winit::event::VirtualKeyCode;

pub struct KeyboardState {
    pub key: [bool; 16],
}

impl KeyboardState {
    pub fn new() -> Self {
        KeyboardState { key: [false; 16] }
    }

    pub fn handle_input(&mut self, key_code: VirtualKeyCode, pressed: bool) {
        match key_code {
            VirtualKeyCode::Key1 => self.key[0x1] = pressed,
            VirtualKeyCode::Key2 => self.key[0x2] = pressed,
            VirtualKeyCode::Key3 => self.key[0x3] = pressed,
            VirtualKeyCode::Key4 => self.key[0xC] = pressed,

            VirtualKeyCode::Q => self.key[0x4] = pressed,
            VirtualKeyCode::W => self.key[0x5] = pressed,
            VirtualKeyCode::E => self.key[0x6] = pressed,
            VirtualKeyCode::R => self.key[0xD] = pressed,

            VirtualKeyCode::A => self.key[0x7] = pressed,
            VirtualKeyCode::S => self.key[0x8] = pressed,
            VirtualKeyCode::D => self.key[0x9] = pressed,
            VirtualKeyCode::F => self.key[0xE] = pressed,

            VirtualKeyCode::Z => self.key[0xA] = pressed,
            VirtualKeyCode::X => self.key[0x0] = pressed,
            VirtualKeyCode::C => self.key[0xB] = pressed,
            VirtualKeyCode::V => self.key[0xF] = pressed,
            _ => (),
        }
    }

    pub fn any_pressed(&self) -> Option<usize> {
        for (i, &state) in self.key.iter().enumerate() {
            if state {
                return Some(i);
            }
        }
        None
    }
}
