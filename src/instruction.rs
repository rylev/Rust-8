use std::fmt;

pub struct Instruction {
    value: u16,
}

impl Instruction {
    pub fn new(value: u16) -> Instruction {
        Instruction { value: value }
    }

    #[inline(always)]
    pub fn xooo(&self) -> u8 {
        ((self.value >> 12) & 0xF) as u8
    }

    #[inline(always)]
    pub fn oxoo(&self) -> u8 {
        ((self.value >> 8) & 0xF) as u8
    }

    #[inline(always)]
    pub fn ooxo(&self) -> u8 {
        ((self.value >> 4) & 0xF) as u8
    }

    #[inline(always)]
    pub fn ooox(&self) -> u8 {
        (self.value as u8) & 0xF
    }

    #[inline(always)]
    pub fn ooxx(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    #[inline(always)]
    pub fn oxxx(&self) -> u16 {
        self.value & 0xFFF
    }
}

impl fmt::LowerHex for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.value)
    }
}
