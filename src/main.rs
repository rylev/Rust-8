use std::env;
use std::fs::File;
use std::io::Read;
use std::fmt;

fn main() {
    let file_name = env::args().nth(1).expect("Must give game name as first file");
    let mut file = File::open(file_name).expect("There was an issue opening the file");
    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data);
    let mut cpu = Cpu::new();
    cpu.run(&game_data);
}

const NUM_GENERAL_PURPOSE_REGS: usize = 15;
const MEMORY_SIZE: usize = 4 * 1024;
const NUM_STACK_FRAMES: usize = 16;

struct Cpu {
    regs: [u8; NUM_GENERAL_PURPOSE_REGS],
    vf_reg: u8,
    i_reg: u16,
    delay_timer_reg: u8,
    sound_timer_reg: u8,
    stack_pointer_reg: u8,
    program_counter_reg: u16,
    memory: [u8; MEMORY_SIZE],
    stack: [u16; NUM_STACK_FRAMES],
}

impl fmt::Debug for Cpu {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "CPU {{ regs: {:?}, i_reg: {}, program_counter_reg: {} }}", self.regs, self.i_reg, self.program_counter_reg)
     }
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            regs: [0; NUM_GENERAL_PURPOSE_REGS],
            vf_reg: 0,
            i_reg: 0,
            delay_timer_reg: 0,
            sound_timer_reg: 0,
            stack_pointer_reg: 0,
            program_counter_reg: 0,
            memory: [0; MEMORY_SIZE],
            stack: [0; NUM_STACK_FRAMES],
        }
    }
}

impl Cpu {
    fn new() -> Cpu {
        Cpu::default()
    }

    fn run(&mut self, game: &Vec<u8>) {
        loop {
            println!("{:?}", self);
            self.run_instruction(game);
            self.program_counter_reg += 2;
        }
    }

    fn run_instruction(&mut self, game: &Vec<u8>) {
        let instruction = self.instruction(game);
        let first = instruction >> 12;
        match first {
            0x6 => {
                let reg_number = ((instruction & 0xF00) >> 8) as u8;
                let value = (instruction & 0xFF) as u8;
                self.load_reg(reg_number, value)
            },
            0xa => {
                let addr = instruction & 0xFFF;
                let value = self.i_reg;
                let lower = value as u8;
                let upper = (value >> 8) as u8;
                self.memory[addr as usize] = upper;
                self.memory[(addr + 1) as usize] = lower;
            },
            _ => panic!("Unrecognized instruction {:x}", instruction)
        }
    }

    fn instruction(&self, game: &Vec<u8>) -> u16 {
        let pc = self.program_counter_reg;
        let higher_order = (game[pc as usize] as u16) << 8;
        let lower_order = game[(pc + 1) as usize] as u16;
        higher_order + lower_order
    }

    fn load_reg(&mut self, reg_number: u8, value: u8) {

        println!("{} {}", reg_number, value);
        self.regs[(reg_number as usize) - 1] = value;
    }
}
