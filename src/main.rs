extern crate sdl2;
extern crate rand;
use std::env;
use std::fs::File;
use std::io::Read;
use std::fmt;

use sdl2::render;
use sdl2::video;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use rand::distributions::{IndependentSample, Range};

fn main() {
    let file_name = env::args().nth(1).expect("Must give game name as first file");
    let mut file = File::open(file_name).expect("There was an issue opening the file");
    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data).expect("Failure to read file");


    let mut computer = Chip8::new(game_data);
    computer.run();
}

const NUM_GENERAL_PURPOSE_REGS: usize = 16;
const MEMORY_SIZE: usize = 4 * 1024;
const NUM_STACK_FRAMES: usize = 16;
const PROGRAM_CODE_OFFSET: usize = 0x200;

struct Chip8<'a> {
    regs: [u8; NUM_GENERAL_PURPOSE_REGS],
    i_reg: u16,
    delay_timer_reg: u8,
    sound_timer_reg: u8,
    stack_pointer_reg: u8,
    program_counter_reg: u16,
    memory: [u8; MEMORY_SIZE],
    stack: [u16; NUM_STACK_FRAMES],
    display: Box<Display<'a>>,
}
impl<'a> Chip8<'a> {
    fn new(program: Vec<u8>) -> Chip8<'a> {
        let mut memory = [0; MEMORY_SIZE];
        //TODO: do this more efficiently
        for (i, byte) in program.iter().enumerate() {
            memory[PROGRAM_CODE_OFFSET + i] = byte.clone();
        }
        for (i, byte) in SPRITES.iter().enumerate() {
            memory[i] = byte.clone();
        }
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let display = Box::new(Display::new(video_subsystem));

        Chip8 {
            regs: [0; NUM_GENERAL_PURPOSE_REGS],
            i_reg: 0,
            delay_timer_reg: 0,
            sound_timer_reg: 0,
            stack_pointer_reg: 0,
            program_counter_reg: PROGRAM_CODE_OFFSET as u16,
            memory: memory,
            stack: [0; NUM_STACK_FRAMES],
            display: display,
        }
    }

    fn run(&mut self) {
        loop {
            let instruction = self.instruction();
            println!("{:x}",instruction.value);
            self.program_counter_reg = self.run_instruction(instruction);
        }
    }

    fn run_instruction(&mut self, instruction: Instruction) -> u16 {
        match instruction.xooo() {
            0x0 => {
                match instruction.ooox() {
                    0xe => {
                        self.stack_pointer_reg -= 1;
                        // TODO: make stack and actual stack
                        self.stack[0] + 2
                    }
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            },
            0x1 => {
                instruction.oxxx()
            }
            0x2 => {
                let addr = instruction.oxxx();
                self.stack_pointer_reg += 1;
                // TODO: make stack and actual stack
                println!("FIX THE SUBROUTINE CALLS");
                self.stack[0] = self.program_counter_reg;
                addr
            },
            0x3 => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                if self.read_reg(reg_number) == value {
                    self.program_counter_reg + 4
                } else {
                    self.program_counter_reg + 2
                }
            },
            0x6 => {
                // load oxoo with the value ooxx
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                self.load_reg(reg_number, value);
                self.program_counter_reg + 2
            },
            0x7 => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                self.load_reg(reg_number, value);
                self.program_counter_reg + 2
            },
            0xa => {
                // load reg i with the value oxxx
                self.i_reg = instruction.oxxx();
                self.program_counter_reg + 2
            },
            0xc => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                let rng = &mut rand::thread_rng();
                let rand_number = Range::new(0,255).ind_sample(rng);
                let and_value = value & rand_number;

                self.load_reg(reg_number, and_value);
                self.program_counter_reg + 2
            },
            0xd => {
                // load ooox bytes to the screen starting at coor oxoo,ooxo with the sprite located
                // at memory location stored in reg i
                let x = instruction.oxoo();
                let y = instruction.ooxo();
                let n = instruction.ooox();
                let i = self.i_reg;
                let from = i as usize;
                let to = from + (n as usize);

                let overwritten = self.display.draw(x, y, &self.memory[from..to]);
                self.regs[0xF] = if overwritten { 1 } else { 0 };
                self.program_counter_reg + 2
            },
            0xF => {
                match instruction.ooxx() {
                    0x7 => {
                        let reg_number = instruction.oxoo();
                        let delay_value = self.delay_timer_reg;
                        self.load_reg(reg_number, delay_value);
                        self.program_counter_reg + 2
                    },
                    0x15 => {
                        //TODO set timer
                        self.program_counter_reg + 2
                    },
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
                    0x65 => {
                        let highest_reg = instruction.oxoo();
                        let i = self.i_reg;
                        let mut offset = 0;
                        for reg_number in 1..(highest_reg + 1) {
                            let value = self.memory[(i + offset) as usize];
                            self.load_reg(reg_number, value);
                            offset += 1;
                        }
                        self.program_counter_reg + 2
                    },
                    _ => panic!("Unrecognized instruction {:x}", instruction.value)
                }
            }
            _ => {
                panic!("Unrecognized instruction {:x}", instruction.value)
            }
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

impl <'a> fmt::Debug for Chip8<'a> {
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
    [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
     0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
     0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
     0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
     0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
     0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
     0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
     0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];

struct Display<'a> {
    renderer: render::Renderer<'a>,
    buffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl<'a> Display<'a> {
    fn window(video_subsystem: sdl2::VideoSubsystem) -> video::Window {
        video_subsystem.window("Rust 8", (DISPLAY_WIDTH * ENLARGEMENT_FACTOR) as u32, (DISPLAY_HEIGHT * ENLARGEMENT_FACTOR) as u32)
            .position_centered().opengl().build().unwrap()
    }

    fn renderer<'b>(window: video::Window) -> render::Renderer<'b> {
        window.renderer().build().unwrap()
    }

    fn new<'b>(video_subsystem: sdl2::VideoSubsystem) -> Display<'b> {
        let window = Display::window(video_subsystem);
        let renderer = Display::renderer(window);
        Display {
            renderer: renderer,
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

                pixel_overwritten = current != new;

                self.buffer[y][x] = new;
            }
        }
        self.flush();
        pixel_overwritten
    }

    fn clear(&mut self) {
        self.renderer.clear();
    }

    fn flush(&mut self) {
        self.clear();
        self.renderer.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));

        for (i, row) in self.buffer.iter().enumerate() {
            print!("|");
            for (j, val) in row.iter().enumerate() {
                if *val { print!("*") } else { print!(" ") }
                if *val {
                    let border_rect = Rect::new(i as i32, j as i32, ENLARGEMENT_FACTOR as u32, ENLARGEMENT_FACTOR as u32).unwrap().unwrap();
                    self.renderer.draw_rect(border_rect);
                    self.renderer.fill_rect(border_rect);
                }
            }
            print!("|\n")
        }

        // sleep(Duration::from_secs(5));
        self.renderer.present();
    }
}

