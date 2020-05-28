const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = WIDTH * HEIGHT;

pub struct DisplayBuffer {
    buffer: [bool; DISPLAY_SIZE],
}

impl DisplayBuffer {
    pub const SIZE: usize = DISPLAY_SIZE;

    pub fn new() -> Self {
        DisplayBuffer {
            buffer: [false; DISPLAY_SIZE],
        }
    }

    pub fn buffer(&self) -> &[bool; DISPLAY_SIZE] {
        &self.buffer
    }

    pub fn write_sprite(&mut self, sprite: &[u8], x: usize, y: usize) -> bool {
        let mut collision = false;

        for (offset_y, &byte) in sprite.iter().enumerate() {
            for (offset_x, &bit) in to_bits(byte).iter().enumerate() {
                if self.set_pos(x + offset_x, y + offset_y, bit) {
                    collision = true;
                }
            }
        }

        collision
    }

    pub fn clear(&mut self) {
        for b in &mut self.buffer[..] {
            *b = false;
        }
    }

    fn set_pos(&mut self, x: usize, y: usize, val: bool) -> bool {
        if x >= WIDTH || y >= HEIGHT {
            return false;
        }

        let index = y * WIDTH + x;

        let collision = self.buffer[index] & val;
        self.buffer[index] ^= val;
        collision
    }
}

fn to_bits(byte: u8) -> [bool; 8] {
    [
        (byte >> 7) == 1,
        ((byte >> 6) & 1) == 1,
        ((byte >> 5) & 1) == 1,
        ((byte >> 4) & 1) == 1,
        ((byte >> 3) & 1) == 1,
        ((byte >> 2) & 1) == 1,
        ((byte >> 1) & 1) == 1,
        (byte & 1) == 1,
    ]
}
