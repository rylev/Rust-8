extern crate sdl2;
use std::env;
use std::fs::File;
use std::io::Read;
use std::fmt;

use sdl2::render;
use sdl2::video;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

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
        for (i, byte) in ZERO_SRITE.iter().enumerate() {
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
            self.run_instruction(instruction);
            self.program_counter_reg += 2;
        }
    }

    fn run_instruction(&mut self, instruction: Instruction) {
        match instruction.xooo() {
            0x6 => {
                let reg_number = instruction.oxoo();
                let value = instruction.ooxx();
                self.load_reg(reg_number, value)
            },
            0xa => {
                let addr = instruction.oxxx();
                let value = self.i_reg;
                let lower = value as u8;
                let upper = (value >> 8) as u8;
                self.memory[addr as usize] = upper;
                self.memory[(addr + 1) as usize] = lower;
            },
            0xd => {
                let x = instruction.oxoo();
                let y = instruction.ooxo();
                let n = instruction.ooox();
                let i = self.i_reg;
                let from = i as usize;
                let to = (i + (n as u16)) as usize;

                let overwritten = self.display.draw(x, y, &self.memory[from..to]);
                self.regs[15] = if overwritten { 1 } else { 0 };
            }
            _ => {
                // loop {}
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

    fn load_reg(&mut self, reg_number: u8, value: u8) {
        self.regs[(reg_number as usize) - 1] = value;
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
        (self.value >> 8) as u8
    }

    #[inline(always)]
    fn oxxx(&self) -> u16 {
        self.value & 0xFFF
    }
}

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 31;
const ENLARGEMENT_FACTOR: usize = 20;
const ZERO_SRITE: [u8; 5] = [0xF0, 0x90, 0x90, 0x90, 0xF0];

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
            let y = ((starting_y - y_offset as u8) % DISPLAY_HEIGHT as u8) as usize;

            for x_offset in 1..8 {
                let bit = (block >> (8 - x_offset)) & 1;
                let x = ((starting_x + x_offset) % DISPLAY_WIDTH as u8) as usize;
                let current = self.buffer[y][x];
                pixel_overwritten = current != (bit == 1);

                self.buffer[y][x] = bit == 1;
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
            for (j, val) in row.iter().enumerate() {
                if *val {
                    println!("{} {}", i, j);
                    let border_rect = Rect::new(i as i32, j as i32, ENLARGEMENT_FACTOR as u32, ENLARGEMENT_FACTOR as u32).unwrap().unwrap();
                    self.renderer.draw_rect(border_rect);
                    self.renderer.fill_rect(border_rect);
                }
            }
        }

        self.renderer.present();
    }
}

