const MEM_SIZE: usize = 4096;
const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Memory {
    bytes: [u8; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        let mut mem = [0; MEM_SIZE];
        mem[0..SPRITES.len()].copy_from_slice(&SPRITES);

        Memory { bytes: mem }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        self.bytes[address]
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        self.bytes[address] = value;
    }

    /// Get the address in memory of the given hexadecimal sprite.
    ///
    /// # Panics
    ///
    /// Panics if the given sprite is outside of 0x0 through 0xF
    pub fn sprite_address(&self, hex_sprite: u8) -> usize {
        match hex_sprite {
            0x0 => 0,
            0x1 => 5,
            0x2 => 10,
            0x3 => 15,
            0x4 => 20,
            0x5 => 25,
            0x6 => 30,
            0x7 => 35,
            0x8 => 40,
            0x9 => 45,
            0xA => 50,
            0xB => 55,
            0xC => 60,
            0xD => 65,
            0xE => 70,
            0xF => 75,
            _ => panic!("invalid sprite: tried to get address of invalid sprite"),
        }
    }

    /// Reads a sprite of `length` bytes, starting at `address`.
    ///
    /// # Panics
    ///
    /// Panics if the given address and length is out of bounds.
    pub fn read_sprite(&self, address: usize, length: usize) -> &[u8] {
        &self.bytes[address..address + length]
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.bytes[0x200..0x200 + rom.len()].copy_from_slice(rom);
    }
}
