pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const SPRITES: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0 /* 0 */, 0x20, 0x60, 0x20, 0x20,
                               0x70 /* 1 */, 0xF0, 0x10, 0xF0, 0x80, 0xF0 /* 2 */, 0xF0,
                               0x10, 0xF0, 0x10, 0xF0 /* 3 */, 0x90, 0x90, 0xF0, 0x10,
                               0x10 /* 4 */, 0xF0, 0x80, 0xF0, 0x10, 0xF0 /* 5 */, 0xF0,
                               0x80, 0xF0, 0x90, 0xF0 /* 6 */, 0xF0, 0x10, 0x20, 0x40,
                               0x40 /* 7 */, 0xF0, 0x90, 0xF0, 0x90, 0xF0 /* 8 */, 0xF0,
                               0x90, 0xF0, 0x10, 0xF0 /* 9 */, 0xF0, 0x90, 0xF0, 0x90,
                               0x90 /* a */, 0xE0, 0x90, 0xE0, 0x90, 0xE0 /* b */, 0xF0,
                               0x80, 0x80, 0x80, 0xF0 /* c */, 0xE0, 0x90, 0x90, 0x90,
                               0xE0 /* d */, 0xF0, 0x80, 0xF0, 0x80, 0xF0 /* e */, 0xF0,
                               0x80, 0xF0, 0x80, 0x80];// f

pub type Buffer = [[bool; WIDTH]; HEIGHT];

pub struct Display {
    buffer: Buffer,
}

impl Display {
    pub fn new() -> Display {
        Display { buffer: [[false; WIDTH]; HEIGHT] }
    }

    pub fn draw(&mut self, starting_x: u8, starting_y: u8, memory: &[u8]) -> bool {
        let mut pixel_turned_off = false;

        for (byte_number, block) in memory.iter().enumerate() {
            let y = (starting_y as usize + byte_number) % HEIGHT;

            for bit_number in 0..8 {
                let x = (starting_x as usize + bit_number) % WIDTH;
                let current_pixel = self.buffer[y][x] as u8;

                let current_bit = (block >> (7 - bit_number)) & 1;
                let new_pixel = current_bit ^ current_pixel;

                self.buffer[y][x] = new_pixel != 0;

                if current_pixel == 1 && new_pixel == 0 {
                    pixel_turned_off = true;
                }
            }
        }
        pixel_turned_off
    }

    pub fn get_buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer = [[false; WIDTH]; HEIGHT];
    }
}
