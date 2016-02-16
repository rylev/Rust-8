use std::fmt;

type Address = u16;
type Register = u8;

enum Instruction {
    ClearDisplay,
    Return,
    Jump(Address),
    Call(Address),
    SkipIfEquals(Register, u8),
    SkipIfNotEquals(Register, u8),
}

struct RawInstruction {
    value: u16,
}

impl RawInstruction {
    pub fn new(value: u16) -> RawInstruction {
        RawInstruction { value: value }
    }

    pub fn to_instruction(&self) -> Option<Instruction> {
        match self.xooo() {
            0x0 => {
                match self.ooxx() {
                    0xEE => Some(ClearDisplay),
                    0xE0 => Some(Return),
                    _    => None
                }
            },
            0x1 => Some(Jump(self.oxxx())),
            0x2 => Some(Call(self.oxxx())),
            0x3 => Some(SkipIfEquals(self.oxoo(), self.ooxx())),
            0x3 => Some(SkipIfNotEquals(self.oxoo(), self.ooxx())),
            _   => None
        }
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
        ((self.value >> 4) & 0xF) as u8
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

impl fmt::LowerHex for RawInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.value)
    }
}
