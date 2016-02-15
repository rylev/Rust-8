extern crate piston_window;
extern crate rand;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;

use piston_window::*;

use rand::distributions::{IndependentSample, Range};

fn main() {
    let file_name = env::args().nth(1).expect("Must give game name as first file");
    let mut file = File::open(file_name).expect("There was an issue opening the file");
    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data).expect("Failure to read file");

    let window_dimensions = [(DISPLAY_WIDTH * ENLARGEMENT_FACTOR) as u32, (DISPLAY_HEIGHT * ENLARGEMENT_FACTOR) as u32];
    let window: PistonWindow = WindowSettings::new("Chip 8 Emulator", window_dimensions).exit_on_esc(true).build().unwrap();
    let mut computer = Chip8::new(game_data);

    for e in window {
        if let Some(_) = e.render_args() {
            computer.display.flush(&e);
        }

        if let Some(u) = e.update_args() {
            computer.cycle(u.dt);
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            if let Some(index) = key_value(&key) {
                computer.keyboard[index as usize] = false;
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if let Some(index) = key_value(&key) {
                computer.keyboard[index as usize] = true;
                if let Some(reg) = computer.wait_for_key {
                    computer.load_reg(reg, index as u8);
                }
                computer.wait_for_key = None;
            }
        }
    }
}

fn key_value(key: &Key) -> Option<i32> {
    if key.code() >= 48 && key.code() <= 57 {
        Some(key.code() - 48)
    } else if key.code() >= 97 && key.code() <= 102 {
        Some(key.code() - 97 + 9)
    } else {
        None
    }
}

const NUM_GENERAL_PURPOSE_REGS: usize = 16;
const MEMORY_SIZE: usize = 4 * 1024;
const NUM_STACK_FRAMES: usize = 16;
const PROGRAM_CODE_OFFSET: usize = 0x200;
const CLOCK_RATE: f64 = 600.0;
const NUM_KEYS: usize = 16;

struct Chip8 {
    regs: [u8; NUM_GENERAL_PURPOSE_REGS],
    i_reg: u16,
    delay_timer_reg: u8,
    sound_timer_reg: u8,
    stack_pointer_reg: u8,
    program_counter_reg: u16,
    memory: [u8; MEMORY_SIZE],
    stack: [u16; NUM_STACK_FRAMES],
    wait_for_key: Option<u8>,
    keyboard: [bool; NUM_KEYS],
    display: Box<Display>,
}
impl Chip8 {
    fn new(program: Vec<u8>) -> Chip8 {
        let mut memory = [0; MEMORY_SIZE];
        //TODO: do this more efficiently
        for (i, byte) in program.iter().enumerate() {
            memory[PROGRAM_CODE_OFFSET + i] = byte.clone();
        }
        for (i, byte) in SPRITES.iter().enumerate() {
            memory[i] = byte.clone();
        }
        let display = Box::new(Display::new());

        Chip8 {
            regs: [0; NUM_GENERAL_PURPOSE_REGS],
            i_reg: 0,
            delay_timer_reg: 0,
            sound_timer_reg: 0,
            stack_pointer_reg: 0,
            program_counter_reg: PROGRAM_CODE_OFFSET as u16,
            memory: memory,
            stack: [0; NUM_STACK_FRAMES],
            wait_for_key: None,
            keyboard: [false; NUM_KEYS],
            display: display,
        }
    }

    fn cycle(&mut self, delta_time: f64) {
        let num_instructions = (delta_time * CLOCK_RATE).round() as u64;
        for _ in 1..num_instructions {
            if self.delay_timer_reg > 0 {
                self.delay_timer_reg -= 1;
            }

            if self.wait_for_key == None{
                let instruction = self.instruction();
                self.program_counter_reg = self.run_instruction(instruction);
            }
        }
    }

    fn run_instruction(&mut self, instruction: Instruction) -> u16 {
        match instruction.xooo() {
            0x0 => {
                match instruction.ooxx() {
                    0xEE => {
                        let addr = self.stack[(self.stack_pointer_reg - 1) as usize];
                        self.stack_pointer_reg -= 1;
                        addr + 2
                    }
                    0xE0 => {
                        self.display.clear();
                        self.program_counter_reg + 2
                    },
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            },
            0x1 => {
                instruction.oxxx()
            }
            0x2 => {
                let addr = instruction.oxxx();
                self.stack_pointer_reg += 1;
                self.stack[(self.stack_pointer_reg - 1) as usize] = self.program_counter_reg;
                addr
            },
            0x3 => {
                let reg_value = self.read_reg(instruction.oxoo());
                let value = instruction.ooxx();
                if reg_value == value {
                    self.program_counter_reg + 4
                } else {
                    self.program_counter_reg + 2
                }
            },
            0x4 => {
                let reg_value = self.read_reg(instruction.oxoo());
                let value = instruction.ooxx();
                if reg_value != value {
                    self.program_counter_reg + 4
                } else {
                    self.program_counter_reg + 2
                }
            },
            0x5 => {
                let first = instruction.oxoo();
                let second = instruction.oxoo();
                if first == second {
                    self.program_counter_reg + 4
                } else {
                    self.program_counter_reg + 2
                }
            }
            0x6 => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                self.load_reg(reg_number, value);
                self.program_counter_reg + 2
            },
            0x7 => {
                let reg_number = instruction.oxoo();
                let reg_value = self.read_reg(reg_number);
                let value = instruction.ooxx().wrapping_add(reg_value);
                self.load_reg(reg_number, value);
                self.program_counter_reg + 2
            },
            0x8 => {
                match instruction.ooox() {
                    0x0 => {
                        let value = self.read_reg(instruction.ooxo());
                        self.load_reg(instruction.oxoo(), value);
                        self.program_counter_reg + 2
                    },
                    0x1 => {
                        panic!("Not yet implemeneted: {:x}", instruction.value);
                    },
                    0x2 => {
                        let first = self.read_reg(instruction.oxoo());
                        let second = self.read_reg(instruction.ooxo());
                        self.load_reg(instruction.oxoo(), first & second);
                        self.program_counter_reg + 2
                    },
                    0x3 => {
                        let first = self.read_reg(instruction.oxoo());
                        let second = self.read_reg(instruction.ooxo());
                        self.load_reg(instruction.oxoo(), first | second);
                        self.program_counter_reg + 2
                    },
                    0x4 => {
                        //8xy4 - ADD Vx, Vy
                        //Set Vx = Vx + Vy, set VF = carry.
                        //The values of Vx and Vy are added together. If the result is greater than
                        //8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits
                        //  of the result are kept, and stored in Vx.
                        let first = self.read_reg(instruction.oxoo()) as u16;
                        let second = self.read_reg(instruction.ooxo()) as u16;
                        let answer = first + second;
                        self.load_reg(0xF, (answer > 255) as u8);
                        self.load_reg(instruction.oxoo(), answer as u8);
                        self.program_counter_reg + 2
                    },
                    0x5 => {
                        let first = self.read_reg(instruction.oxoo());
                        let second = self.read_reg(instruction.ooxo());
                        self.load_reg(0xF, (first > second) as u8);
                        self.load_reg(instruction.oxoo(), first.wrapping_sub(second));
                        self.program_counter_reg + 2
                    },
                    0x6 => {
                        let value = self.read_reg(instruction.ooxo());
                        self.load_reg(0xF, value & 0b1);
                        self.load_reg(instruction.oxoo(), value >> 1);
                        self.program_counter_reg + 2
                    },
                    0x7 => {
                        panic!("Not yet implemeneted: {:x}", instruction.value);
                    },
                    0xE => {
                        let value = self.read_reg(instruction.ooxo());
                        self.load_reg(0xF, value >> 7);
                        self.load_reg(instruction.oxoo(), value << 1);
                        self.program_counter_reg + 2
                    },
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            },
            0x9 => {
                let first = instruction.oxoo();
                let second = instruction.oxoo();
                if first != second {
                    self.program_counter_reg + 4
                } else {
                    self.program_counter_reg + 2
                }
            },
            0xA => {
                // load reg i with the value oxxx
                self.i_reg = instruction.oxxx();
                self.program_counter_reg + 2
            },
            0xB => {
                panic!("Not yet implemeneted: {:x}", instruction.value);
            },
            0xC => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                let rng = &mut rand::thread_rng();
                let rand_number = Range::new(0,255).ind_sample(rng);

                self.load_reg(reg_number, rand_number & value);
                self.program_counter_reg + 2
            },
            0xD => {
                // load ooox bytes to the screen starting at coor oxoo,ooxo with the sprite located
                // at memory location stored in reg i
                let x = self.read_reg(instruction.oxoo());
                let y = self.read_reg(instruction.ooxo());
                let n = instruction.ooox();
                let i = self.i_reg;
                let from = i as usize;
                let to = from + (n as usize);

                self.regs[0xF] = self.display.draw(x, y, &self.memory[from..to]) as u8;
                self.program_counter_reg + 2
            },
            0xE => {
                let value = self.read_reg(instruction.oxoo());
                let pressed = self.keyboard[value as usize];
                match instruction.ooxx() {
                    0x9E => {
                        if pressed {
                            self.program_counter_reg + 4
                        } else {
                            self.program_counter_reg + 2
                        }
                    },
                    0xA1 => {
                        if pressed {
                            self.program_counter_reg + 2
                        } else {
                            self.program_counter_reg + 4
                        }
                    },
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            },
            0xF => {
                match instruction.ooxx() {
                    0x07 => {
                        let reg_number = instruction.oxoo();
                        let delay_value = self.delay_timer_reg;
                        self.load_reg(reg_number, delay_value);
                        self.program_counter_reg + 2
                    },
                    0x0A => {
                        self.wait_for_key = Some(instruction.oxoo());
                        self.program_counter_reg + 2
                    }
                    0x15 => {
                        let value = self.read_reg(instruction.oxoo());
                        self.delay_timer_reg = value;
                        self.program_counter_reg + 2
                    },
                    0x18 => {
                        //TODO: set sound timer
                        self.program_counter_reg + 2
                    },
                    0x1E => {
                        let value = self.read_reg(instruction.oxoo()) as u16;
                        self.i_reg = self.i_reg + value;
                        self.program_counter_reg + 2
                    }
                    0x29 => {
                        let reg = instruction.oxoo();
                        let digit = self.read_reg(reg);
                        self.i_reg = (digit * 5) as u16;
                        self.program_counter_reg + 2
                    },
                    0x33 => {
                        let reg_number = instruction.oxoo();
                        let value = self.read_reg(reg_number);
                        self.memory[self.i_reg as usize] = (value / 100) % 10;
                        self.memory[(self.i_reg + 1) as usize] = (value / 10) % 10;
                        self.memory[(self.i_reg + 2) as usize] = value % 10;
                        self.program_counter_reg + 2
                    },
                    0x55 => {
                        let highest_reg = instruction.oxoo();

                        let i = self.i_reg;
                        for reg_number in 0..(highest_reg + 1) {
                            self.memory[(i + reg_number as u16) as usize] = self.read_reg(reg_number);
                        }
                        self.program_counter_reg + 2
                    }
                    0x65 => {
                        let highest_reg = instruction.oxoo();
                        let i = self.i_reg;
                        for reg_number in 0..(highest_reg + 1) {
                            let value = self.memory[(i + reg_number as u16) as usize];
                            self.load_reg(reg_number, value);
                        }
                        self.program_counter_reg + 2
                    },
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            },
            _ => panic!("Unrecognized instruction {:x}", instruction.value)
        }
    }

    fn instruction(&self) -> Instruction {
        let pc = self.program_counter_reg;
        let higher_order = (self.memory[pc as usize] as u16) << 8;
        let lower_order = self.memory[(pc + 1) as usize] as u16;
        Instruction::new(higher_order + lower_order)
    }

    fn read_reg(&self, reg_number: u8) -> u8 {
        self.regs[(reg_number as usize)]
    }

    fn load_reg(&mut self, reg_number: u8, value: u8) {
        self.regs[(reg_number as usize)] = value;
    }
}

impl<'a> fmt::Debug for Chip8 {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "CPU {{ regs: {:?}, i_reg: {}, program_counter_reg: {} }}", self.regs, self.i_reg, self.program_counter_reg)
     }
}

struct Instruction{
    value: u16
}

impl Instruction {
    fn new(value: u16) -> Instruction {
        Instruction{value: value}
    }

    #[inline(always)]
    fn xooo(&self) -> u8 {
        ((self.value >> 12) & 0xF) as u8
    }

    #[inline(always)]
    fn oxoo(&self) -> u8 {
        ((self.value >> 8) & 0xF) as u8
    }

    #[inline(always)]
    fn ooxo(&self) -> u8 {
        ((self.value >> 4) & 0xF)  as u8
    }

    #[inline(always)]
    fn ooox(&self) -> u8 {
        (self.value as u8) & 0xF
    }

    #[inline(always)]
    fn ooxx(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    #[inline(always)]
    fn oxxx(&self) -> u16 {
        self.value & 0xFFF
    }
}

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const ENLARGEMENT_FACTOR: usize = 20;
const SPRITES: [u8; 80] =
    [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
     0x20, 0x60, 0x20, 0x20, 0x70, // 1
     0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
     0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
     0x90, 0x90, 0xF0, 0x10, 0x10, // 4
     0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
     0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
     0xF0, 0x10, 0x20, 0x40, 0x40, // 7
     0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
     0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
     0xF0, 0x90, 0xF0, 0x90, 0x90, // a
     0xE0, 0x90, 0xE0, 0x90, 0xE0, // b
     0xF0, 0x80, 0x80, 0x80, 0xF0, // c
     0xE0, 0x90, 0x90, 0x90, 0xE0, // d
     0xF0, 0x80, 0xF0, 0x80, 0xF0, // e
     0xF0, 0x80, 0xF0, 0x80, 0x80];// f

struct Display {
    buffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display {

    fn new() -> Display {
        Display {
            buffer: [[false;DISPLAY_WIDTH];DISPLAY_HEIGHT]
        }
    }

    fn draw(&mut self, starting_x: u8, starting_y: u8, memory: &[u8]) -> bool {
        let mut pixel_overwritten = false;
        for (y_offset, block) in memory.iter().enumerate() {
            let y = ((starting_y + y_offset as u8) % DISPLAY_HEIGHT as u8) as usize;

            for x_offset in 0..8 {
                let x = ((starting_x + x_offset) % DISPLAY_WIDTH as u8) as usize;
                let current = self.buffer[y][x];

                let bit = (block >> (7 - x_offset)) & 1 == 1;
                let new = bit ^ current;

                self.buffer[y][x] = new;

                if current && !new {
                    pixel_overwritten = true;
                }
            }
        }
        pixel_overwritten
    }

    fn clear(&mut self) {
        self.buffer = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    fn flush(&mut self, window: &PistonWindow) {
        window.draw_2d(|context, graphics| {
            piston_window::clear(color::BLACK, graphics);

            for (i, row) in self.buffer.iter().enumerate() {
                for (j, val) in row.iter().enumerate() {
                    if *val {
                        let dimensions = [(j * ENLARGEMENT_FACTOR) as f64, (i * ENLARGEMENT_FACTOR) as f64, ENLARGEMENT_FACTOR as f64, ENLARGEMENT_FACTOR as f64];
                        Rectangle::new(color::WHITE).draw(dimensions, &context.draw_state, context.transform, graphics);
                    }
                }
            }
        })
    }

    #[allow(dead_code)]
    fn debug(&mut self) {
        for row in self.buffer.iter() {
            print!("|");
            for val in row.iter() {
                if *val { print!("*") } else { print!(".") }
            }
            println!("|")
        }
    }
}

