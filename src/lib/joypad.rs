pub(crate) struct Joypad {
    pub(crate) interrupt: u8,
    buttons: u8,
    reg: u8,
}

impl Joypad {
    pub(crate) fn new() -> Self {
        Self {
            interrupt: 0,
            buttons: 0xFF,
            reg: 0,
        }
    }

    pub(crate) fn keydown(&mut self, key: u8) {
        self.buttons &= !key;
        self.interrupt |= 0x10;
    }

    pub(crate) fn keyup(&mut self, key: u8) {
        self.buttons |= key;
    }

    pub(crate) fn rb(&self, _a: u16) -> u8 {
        if (self.reg & 0b0001_0000) == 0 {
            return self.reg | (self.buttons & 0x0F);
        }

        if (self.reg & 0b0010_0000) == 0 {
            return self.reg | (self.buttons >> 4);
        }

        self.reg
    }

    pub(crate) fn wb(&mut self, _a: u16, v: u8) {
        self.reg = v;
    }
}
