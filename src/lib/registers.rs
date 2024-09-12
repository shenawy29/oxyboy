use crate::cpu::Flag;

#[derive(Default, Debug)]
pub(crate) struct Registers {
    pub(crate) a: u8,
    pub(crate) f: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
}

impl Registers {
    pub(crate) fn decode_r8(&self, r: u8) -> Register {
        match r {
            0 => Register::B,
            1 => Register::C,
            2 => Register::D,
            3 => Register::E,
            4 => Register::H,
            5 => Register::L,
            6 => Register::Hlm,
            7 => Register::A,
            _ => unreachable!(),
        }
    }

    pub(crate) fn decode_r16_g1(&self, r: u8) -> Register {
        match r {
            0 => Register::BC,
            1 => Register::DE,
            2 => Register::HL,
            3 => Register::SP,
            _ => unreachable!(),
        }
    }

    pub(crate) fn decode_r16_g2(&self, r: u8) -> Register {
        match r {
            0 => Register::BC,
            1 => Register::DE,
            2 => Register::Hli,
            3 => Register::Hld,
            _ => unreachable!(),
        }
    }

    pub(crate) fn decode_r16_g3(&self, r: u8) -> Register {
        match r {
            0 => Register::BC,
            1 => Register::DE,
            2 => Register::HL,
            3 => Register::AF,
            _ => unreachable!(),
        }
    }

    pub(crate) fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub(crate) fn hld(&mut self) -> u16 {
        let hl = self.hl();
        self.sethl(hl.wrapping_sub(1));
        hl
    }

    pub(crate) fn hli(&mut self) -> u16 {
        let hl = self.hl();
        self.sethl(hl.wrapping_add(1));
        hl
    }

    pub(crate) fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16 & 0x00F0)
    }

    pub(crate) fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub(crate) fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub(crate) fn sethl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0x00FF) as u8;
    }
    pub(crate) fn setbc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0x00FF) as u8;
    }
    pub(crate) fn setde(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0x00FF) as u8;
    }
    pub(crate) fn setaf(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0x00F0) as u8;
    }

    pub(crate) fn getflag(&mut self, flag: Flag) -> bool {
        let mask = flag as u8;
        self.f & mask > 0
    }

    pub(crate) fn flag(&mut self, flags: Flag, set: bool) {
        let mask = flags as u8;
        match set {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
        self.f &= 0xF0;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub(crate) enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    // Memory register.
    Hlm,
    // [HL+]
    Hli,
    // [HL-]
    Hld,
    SP,
}
