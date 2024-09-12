#[derive(Default)]
pub struct Timer {
    internaldiv: u32,
    internalcnt: u32,
    counter: u8,
    modulo: u8,
    pub interrupt: u8,
    control: u8,
}

impl Timer {
    #[inline(always)]
    pub fn rb(&self, a: u16) -> u8 {
        match a {
            0xFF04 => (self.internaldiv >> 8) as u8,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn wb(&mut self, a: u16, v: u8) {
        match a {
            0xFF04 => self.internaldiv = 0,
            0xFF05 => self.counter = v,
            0xFF06 => self.modulo = v,
            0xFF07 => self.control = v,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn do_cycle(&mut self, c: u32) {
        self.internaldiv += c;

        if self.control & 0x4 != 0 {
            self.internalcnt += c;

            let step = match self.control & 0x3 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!(),
            };

            while self.internalcnt >= step {
                self.counter = self.counter.wrapping_add(1);

                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt |= 0x04;
                }

                self.internalcnt -= step;
            }
        }
    }
}
