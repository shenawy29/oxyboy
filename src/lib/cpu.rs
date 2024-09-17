use crate::{
    mmu::Mmu,
    registers::{Register, Registers},
};

use bitmatch::bitmatch;

use Flag::{C, H, N, Z};

#[derive(Debug)]
enum Condition {
    NZ,
    Z,
    NC,
    C,
}

impl Condition {
    pub(crate) fn check(&self, context: &Cpu) -> bool {
        let z = context.check_flag(Flag::Z);
        let c = context.check_flag(Flag::C);

        match self {
            Self::C => c,
            Self::NC => !c,
            Self::Z => z,
            Self::NZ => !z,
        }
    }
}

#[repr(u8)]
pub(crate) enum Flag {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

pub(crate) struct Cpu {
    pub(crate) ime: bool,
    pub(crate) mmu: Mmu,
    setei: u8,
    setdi: u8,
    pub(crate) reg: Registers,
    pub(crate) halted: bool,
}

fn decode_condition(cond: u8) -> Condition {
    match cond {
        0 => Condition::NZ,
        1 => Condition::Z,
        2 => Condition::NC,
        3 => Condition::C,
        _ => unreachable!(),
    }
}

impl Cpu {
    pub fn get_gpu_data(&mut self) -> &[u8] {
        &self.mmu.ppu.buffer
    }

    pub(crate) fn new() -> Self {
        Self {
            mmu: Mmu::new(),
            reg: Registers {
                pc: 0x100,
                sp: 0xFFFE,
                a: 0x1,
                f: 0x80,
                b: 0xFF,
                c: 0x13,
                d: 0x0,
                e: 0xC1,
                h: 0x84,
                l: 0x03,
            },
            ime: false,
            setei: 0,
            setdi: 0,
            halted: false,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.reg = Registers {
            pc: 0x100,
            sp: 0xFFFE,
            a: 0x1,
            f: 0x80,
            b: 0xFF,
            c: 0x13,
            d: 0x0,
            e: 0xC1,
            h: 0x84,
            l: 0x03,
        };
        self.ime = false;

        self.setei = 0;
        self.setdi = 0;
        self.halted = false;
    }

    #[rustfmt::skip]
    #[bitmatch]
    pub fn fde(&mut self) -> u32 {
        let opcode = self.fetchb();

        #[bitmatch]
        match opcode {
            "00000000" => 1, // 0x0
            "00001000" => { let v = self.fetchw(); self.writew(v, self.reg.sp);  3 }
            "00010000" => 1,
            "00011000" => { self.jr(); 3 }
            "001cc000" => {
                if decode_condition(c).check(self) { self.jr(); 3 }
                else { self.reg.pc = self.reg.pc.wrapping_add(1); 2 }
            }
            "00rr0001" => { let r = self.reg.decode_r16_g1(r); let v = self.fetchw(); self.setreg(&r, v); 3 }
            "00rr1001" => {
                let r = self.reg.decode_r16_g1(r);
                let v = self.readreg(&r);
                self.alu_add16(v);
                2
            }
            "00rr0010" => { let r = self.reg.decode_r16_g2(r); let a = self.readreg(&r); let v = self.reg.a; self.writeb(a, v); 2 }
            "00rr1010" => {
                let r = self.reg.decode_r16_g2(r);
                let a = self.readreg(&r);
                let v = self.mmu.rb(a);
                self.reg.a = v;
                2
            }
            "00rr0011" => { let r = self.reg.decode_r16_g1(r); let v = self.readreg(&r); self.setreg(&r, v.wrapping_add(1)); 2 }
            "00rr1011" => { let r = self.reg.decode_r16_g1(r); let v = self.readreg(&r); self.setreg(&r, v.wrapping_sub(1)); 2 }
            "00rrr100" => {
                let r = self.reg.decode_r8(r);
                let a = self.readreg(&r);
                match r {
                    Register::Hlm => {
                        let a = self.reg.hl();
                        let v = self.mmu.rb(a);
                        let v2 = self.alu_inc(v);
                        self.mmu.wb(a, v2);
                        3
                    }
                    _ => {
                        let v = self.alu_inc(a as u8);
                        self.setreg(&r, v as u16);
                        1
                    }
                }
            }

            "00rrr101" => {
                let r = self.reg.decode_r8(r);
                let a = self.readreg(&r);
                match r {
                    Register::Hlm => {
                        let a = self.reg.hl();
                        let v = self.mmu.rb(a);
                        let v2 = self.alu_dec(v);
                        self.mmu.wb(a, v2);
                        3
                    }
                    _ => {
                        let v = self.alu_dec(a as u8);
                        self.setreg(&r, v as u16);
                        1
                    }
                }
            }
            "00rrr110" => {
                let r = self.reg.decode_r8(r);
                match r {
                    Register::Hlm => { let v = self.fetchb(); let a = self.readreg(&r); self.writeb(a, v); 3 }
                    _ => { let v = self.fetchb(); self.setreg(&r, v as u16); 2 }
                }
            }
            "00iii111" => match i {
                0 => { self.reg.a = self.alu_rlc(self.reg.a); self.reg.flag(Z, false); 1},
                1 => { self.reg.a = self.alu_rrc(self.reg.a); self.reg.flag(Z, false); 1}
                2 => { self.reg.a = self.alu_rl(self.reg.a); self.reg.flag(Z, false); 1 }
                3 => { self.reg.a = self.alu_rr(self.reg.a); self.reg.flag(Z, false); 1}
                4 => { self.alu_daa(); 1},
                5 => { self.reg.a = !self.reg.a; self.reg.flag(H, true); self.reg.flag(N, true); 1}
                6 => { self.reg.flag(C, true); self.reg.flag(H, false); self.reg.flag(N, false); 1}
                7 => { let c = !self.reg.getflag(C); self.reg.flag(C, c); self.reg.flag(H, false); self.reg.flag(N, false); 1}
                _ => unreachable!(),
            },
            "01110110" => { self.halted = true; 1 }
            "01dddsss" => {
                let s = &self.reg.decode_r8(s);
                let d = &self.reg.decode_r8(d);

                if d == &Register::Hlm {
                    let a = self.reg.hl();
                    let v = self.readreg(s) as u8;
                    self.writeb(a, v);
                    3
                } else if s == &Register::Hlm {
                    let a = self.reg.hl();
                    let v = self.mmu.rb(a);
                    self.setreg(d, v as u16);
                    3
                } else {
                    let v = self.readreg(s);
                    self.setreg(d, v);
                    2
                }
            }

            "10iiirrr" => {
                let r = self.reg.decode_r8(r);
                let v = self.readr8reg(&r);
                self.alu_a(i, v);
                if r == Register::Hlm { 3 } else { 2 }
            }

            "110cc000" => {
                let c = decode_condition(c).check(self);
                if c { self.reg.pc = self.popstack(); 4 } else { 2 }
            }

            "11100000" => {
                let a = 0xFF00 | self.fetchb() as u16;
                self.writeb(a, self.reg.a);
                3
            }

            "11101000" => {
                let b = self.fetchb() as i8 as i16 as u16;
                let a = self.reg.sp;

                self.reg.flag(N, false);
                self.reg.flag(Z, false);
                self.reg.flag(H, (a & 0x000F) + (b & 0x000F) > 0x000F);
                self.reg.flag(C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);

                self.reg.sp = a.wrapping_add(b);
                4
            }

            "11110000" => { let a = self.fetchb() as u16 | 0xFF00; let v = self.mmu.rb(a); self.reg.a = v; 3 }

            "11111000" => {
                let v = self.fetchb() as i8 as i16 as u16;
                let sp = self.reg.sp;

                self.reg.flag(N, false);
                self.reg.flag(Z, false);
                self.reg.flag(H, (sp & 0x000F) + (v & 0x000F) > 0x000F);
                self.reg.flag(C, (sp & 0x00FF) + (v & 0x00FF) > 0x00FF);

                self.reg.sethl(sp.wrapping_add(v));
                3
            }

            "11rr0001" => {
                let r = self.reg.decode_r16_g3(r);
                let mut v = self.popstack();
                if r == Register::A { v &= 0xFFF0 }
                self.setreg(&r, v);
                3
            }

            "11ii1001" => match i {
                0 => { self.reg.pc = self.popstack(); 4 },
                1 => { self.setei = 1; self.reg.pc = self.popstack(); 4 }
                2 => { self.reg.pc = self.reg.hl(); 1 },
                3 => { self.reg.sp = self.reg.hl(); 2 },
                _ => unreachable!(),
            },

            "110cc010" => match decode_condition(c).check(self) {
                true => { self.reg.pc = self.fetchw(); 4 }
                false => { self.reg.pc = self.reg.pc.wrapping_add(2); 3 }
            },

            "11100010" => { let a = self.reg.c as u16 | 0xFF00; let v = self.reg.a; self.writeb(a, v); 2 }
            "11101010" => { let a = self.fetchw(); self.writeb(a, self.reg.a); 4 }
            "11110010" => { let a = self.reg.c as u16 | 0xFF00; let v = self.mmu.rb(a); self.reg.a = v; 2 }
            "11111010" => { let a = self.fetchw(); let v = self.mmu.rb(a); self.reg.a = v; 4 }

            "11iii011" => {
                match i {
                    0 => { let x = self.fetchw(); self.reg.pc = x; 4 }
                    1 => {
                        let opcode = self.fetchb();

                        let after_prefix =
                        #[bitmatch]
                        match opcode {
                            "00iiirrr" => {
                                let r = self.reg.decode_r8(r);
                                self.alu_shift(i, &r);
                                if r == Register::Hlm { 4 } else { 2 }
                            }
                            "01bbbrrr" => {
                                let r = self.reg.decode_r8(r);
                                let v = self.readr8reg(&r);
                                self.alu_bit(v, b);
                                if r == Register::Hlm { 3 } else { 2 }
                            }
                            "10bbbrrr" => {
                                let r = self.reg.decode_r8(r);
                                self.alu_res(&r, b);
                                if r == Register::Hlm { 4 } else { 2 }
                            }
                            "11bbbrrr" => {
                                let r = self.reg.decode_r8(r);
                                let v = self.readreg(&r);

                                if r == Register::Hlm {
                                    let a = v;
                                    let v = self.mmu.rb(a);
                                    self.writeb(a, v | 1 << b);
                                    4
                                } else {
                                    self.setreg(&r, v | 1 << b);
                                    2
                                }
                            }
                            _ => unreachable!()
                        };

                        1 + after_prefix
                    }
                    6 => { self.setdi = 1; 1 },
                    7 => { self.setei = 2; 1 },
                    _ => unreachable!(),
                }
            },
            "110cc100" => { let c = decode_condition(c); self.conditional_call(c) }
            "11rr0101" => { let r = self.reg.decode_r16_g3(r); let v = self.readreg(&r); self.pushstack(v); 4 }
            "11001101" => { self.pushstack(self.reg.pc + 2); self.reg.pc = self.fetchw(); 6 }
            "11iii110" => { let v = self.fetchb(); self.alu_a(i, v); 2}
            "11eee111" => {
                self.pushstack(self.reg.pc);
                self.reg.pc = (e << 3) as u16;
                4
            }
            _ => unreachable!(),
        }
    }

    fn updateime(&mut self) {
        self.setdi = match self.setdi {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };

        self.setei = match self.setei {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    fn alu_shift(&mut self, i: u8, r: &Register) {
        let v = self.readreg(r);

        match i {
            0 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_rlc(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_rlc(v as u8);
                self.setreg(r, res as u16);
            }
            1 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_rrc(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_rrc(v as u8);
                self.setreg(r, res as u16);
            }
            2 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_rl(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_rl(v as u8);
                self.setreg(r, res as u16);
            }
            3 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_rr(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_rr(v as u8);
                self.setreg(r, res as u16);
            }
            4 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_sla(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_sla(v as u8);
                self.setreg(r, res as u16);
            }
            5 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_sra(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_sra(v as u8);
                self.setreg(r, res as u16);
            }
            6 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_swap(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_swap(v as u8);
                self.setreg(r, res as u16);
            }
            7 => {
                if r == &Register::Hlm {
                    let a = v;
                    let val = self.mmu.rb(a);
                    let res = self.alu_srl(val);
                    return self.writeb(a, res);
                }

                let res = self.alu_srl(v as u8);
                self.setreg(r, res as u16);
            }
            _ => unreachable!(),
        }
    }

    fn conditional_call(&mut self, cond: Condition) -> u32 {
        match cond {
            Condition::Z => {
                if self.reg.getflag(Z) {
                    self.pushstack(self.reg.pc + 2);
                    self.reg.pc = self.fetchw();
                    6
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                    3
                }
            }
            Condition::NZ => {
                if !self.reg.getflag(Z) {
                    self.pushstack(self.reg.pc + 2);
                    self.reg.pc = self.fetchw();
                    6
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                    3
                }
            }
            Condition::NC => {
                if !self.reg.getflag(C) {
                    self.pushstack(self.reg.pc + 2);
                    self.reg.pc = self.fetchw();
                    6
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                    3
                }
            }
            Condition::C => {
                if self.reg.getflag(C) {
                    self.pushstack(self.reg.pc + 2);
                    self.reg.pc = self.fetchw();
                    6
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                    3
                }
            }
        }
    }

    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.reg.flag(Z, a == 0);
        self.reg.flag(C, false);
        self.reg.flag(H, false);
        self.reg.flag(N, false);
        a.rotate_left(4)
    }

    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = a << 1;
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_res(&mut self, r: &Register, b: u8) {
        use Register::{Hlm, A, B, C, D, E, H, L};
        let v = self.readreg(r);

        match r {
            A | B | C | D | E | H | L => self.setreg(r, v & !(1 << b)),
            Hlm => {
                let res = self.mmu.rb(v);
                self.writeb(v, res & !(1 << b))
            }
            _ => unreachable!(),
        };
    }

    fn alu_bit(&mut self, r: u8, b: u8) {
        let v = r & (1 << (b as u32)) == 0;
        self.reg.flag(N, false);
        self.reg.flag(H, true);
        self.reg.flag(Z, v);
    }

    fn popstack(&mut self) -> u16 {
        let res = self.mmu.rw(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        res
    }

    fn pushstack(&mut self, value: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.writew(self.reg.sp, value);
    }

    fn jr(&mut self) {
        let n = self.fetchb() as i8;
        self.reg.pc = ((self.reg.pc as u32 as i32) + (n as i32)) as u16;
    }

    fn writew(&mut self, a: u16, v: u16) {
        self.mmu.ww(a, v)
    }

    fn writeb(&mut self, a: u16, v: u8) {
        self.mmu.wb(a, v)
    }

    fn fetchw(&mut self) -> u16 {
        let w = self.mmu.rw(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        w
    }

    fn fetchb(&mut self) -> u8 {
        let b = self.mmu.rb(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        b
    }

    pub fn check_flag(&self, flag: Flag) -> bool {
        match flag {
            Z => self.reg.f & Z as u8 != 0,
            N => self.reg.f & N as u8 != 0,
            H => self.reg.f & H as u8 != 0,
            C => self.reg.f & C as u8 != 0,
        }
    }

    pub(crate) fn step(&mut self) -> u32 {
        self.updateime();

        match self.handle_interrupt() {
            0 => {}
            n => return n,
        };

        if self.halted {
            1
        } else {
            self.fde()
        }
    }

    pub(crate) fn docycle(&mut self) {
        let cycle = self.step() * 4;
        self.mmu.tick(cycle);
    }

    fn handle_interrupt(&mut self) -> u32 {
        if !self.ime && !self.halted {
            return 0;
        }

        let triggered = self.mmu.inte & self.mmu.intf;

        if triggered == 0 {
            return 0;
        }

        self.halted = false;

        if !self.ime {
            return 0;
        }

        self.ime = false;

        let n = triggered.trailing_zeros();

        self.mmu.intf &= !(1 << n);

        let pc = self.reg.pc;

        self.pushstack(pc);

        self.reg.pc = 0x0040 | ((n as u16) << 3);

        4
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.reg.flag(Z, r == 0);
        self.reg.flag(H, (a & 0x0F) + 1 > 0x0F);
        self.reg.flag(N, false);
        r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.reg.flag(Z, r == 0);
        self.reg.flag(H, (a & 0x0F) == 0);
        self.reg.flag(N, true);
        r
    }

    fn alu_add16(&mut self, b: u16) {
        let a = self.reg.hl();
        let r = a.wrapping_add(b);

        self.reg.flag(H, ((a & 0x0FFF) + (b & 0x0FFF)) > 0x0FFF);
        self.reg.flag(N, false);
        self.reg.flag(C, a > 0xFFFF - b);
        self.reg.sethl(r);
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.reg.getflag(C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_add(b).wrapping_add(c);
        self.reg.flag(Z, r == 0);
        self.reg.flag(H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.reg.flag(N, false);
        self.reg
            .flag(C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.reg.a = r;
    }

    fn alu_sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.reg.getflag(C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.reg.flag(Z, r == 0);
        self.reg.flag(H, (a & 0x0F) < (b & 0x0F) + c);
        self.reg.flag(N, true);
        self.reg.flag(C, (a as u16) < (b as u16) + (c as u16));
        self.reg.a = r;
    }

    fn readr8reg(&mut self, r: &Register) -> u8 {
        use Register::{Hlm, A, B, C, D, E, H, L};

        match r {
            A | B | C | D | E | H | L => self.readreg(r) as u8,
            Hlm => self.mmu.rb(self.reg.hl()),
            _ => unreachable!(),
        }
    }

    fn alu_a(&mut self, i: u8, v: u8) {
        match i {
            0 => self.alu_add(v, false),
            1 => self.alu_add(v, true),
            2 => self.alu_sub(v, false),
            3 => self.alu_sub(v, true),
            4 => self.alu_and(v),
            5 => self.alu_xor(v),
            6 => self.alu_or(v),
            7 => self.alu_cp(v),
            _ => unreachable!(),
        };
    }

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.reg.flag(H, false);
        self.reg.flag(N, false);
        self.reg.flag(Z, r == 0);
        self.reg.flag(C, c);
    }

    fn alu_and(&mut self, b: u8) {
        let r = self.reg.a & b;
        self.reg.flag(Z, r == 0);
        self.reg.flag(H, true);
        self.reg.flag(C, false);
        self.reg.flag(N, false);
        self.reg.a = r;
    }

    fn alu_or(&mut self, b: u8) {
        let r = self.reg.a | b;
        self.reg.flag(Z, r == 0);
        self.reg.flag(C, false);
        self.reg.flag(H, false);
        self.reg.flag(N, false);
        self.reg.a = r;
    }

    fn alu_xor(&mut self, b: u8) {
        let r = self.reg.a ^ b;
        self.reg.flag(Z, r == 0);
        self.reg.flag(C, false);
        self.reg.flag(H, false);
        self.reg.flag(N, false);
        self.reg.a = r;
    }

    fn alu_cp(&mut self, b: u8) {
        let r = self.reg.a;
        self.alu_sub(b, false);
        self.reg.a = r;
    }

    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.reg.getflag(C) { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.reg.getflag(C) { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        r
    }

    fn alu_daa(&mut self) {
        let mut a = self.reg.a;

        let mut adjust = if self.reg.getflag(C) { 0x60 } else { 0x00 };

        if self.reg.getflag(H) {
            adjust |= 0x06;
        };

        if !self.reg.getflag(N) {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.reg.flag(C, adjust >= 0x60);
        self.reg.flag(H, false);
        self.reg.flag(Z, a == 0);
        self.reg.a = a;
    }

    fn setreg(&mut self, reg: &Register, val: u16) {
        match reg {
            Register::A => self.reg.a = val as u8,
            Register::B => self.reg.b = val as u8,
            Register::C => self.reg.c = val as u8,
            Register::D => self.reg.d = val as u8,
            Register::E => self.reg.e = val as u8,
            Register::H => self.reg.h = val as u8,
            Register::L => self.reg.l = val as u8,

            Register::AF => self.reg.setaf(val),
            Register::BC => self.reg.setbc(val),
            Register::DE => self.reg.setde(val),
            Register::HL => self.reg.sethl(val),
            Register::Hli => self.reg.sethl(val),
            Register::Hld => self.reg.sethl(val),
            Register::Hlm => unreachable!(),
            Register::SP => self.reg.sp = val,
        }
    }

    fn readreg(&mut self, reg: &Register) -> u16 {
        match reg {
            Register::A => self.reg.a as u16,
            Register::B => self.reg.b as u16,
            Register::C => self.reg.c as u16,
            Register::D => self.reg.d as u16,
            Register::E => self.reg.e as u16,
            Register::H => self.reg.h as u16,
            Register::L => self.reg.l as u16,

            Register::AF => self.reg.af(),
            Register::HL => self.reg.hl(),
            Register::Hli => self.reg.hli(),
            Register::Hld => self.reg.hld(),
            Register::Hlm => self.reg.hl(),
            Register::BC => self.reg.bc(),
            Register::DE => self.reg.de(),
            Register::SP => self.reg.sp,
        }
    }
}
