use std::path::PathBuf;

use crate::{cartridge::Cartridge, joypad::Joypad, ppu::Ppu, timer::Timer};

pub(crate) struct Mmu {
    pub(crate) cart: Cartridge,
    pub(crate) timer: Timer,
    pub(crate) ppu: Ppu,
    pub(crate) joypad: Joypad,
    pub(crate) serial_data: [u8; 2],
    pub(crate) inte: u8,
    pub(crate) intf: u8,
}

impl Mmu {
    pub(crate) fn new() -> Self {
        Self {
            cart: Cartridge::new(),
            serial_data: [0, 0],
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            timer: Timer::default(),
            inte: 0,
            intf: 0,
        }
    }

    pub(crate) fn from(cart: PathBuf) -> Self {
        Self {
            cart: Cartridge::from(cart),
            serial_data: [0, 0],
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            timer: Timer::default(),
            inte: 0,
            intf: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn rb(&self, a: u16) -> u8 {
        match a {
            0x0000..0x8000 => self.cart.mbc.read(a),
            0x8000..0xA000 => self.ppu.rb(a),
            0xA000..0xC000 => self.cart.mbc.read(a),
            0xC000..0xE000 => self.cart.wram_read(a),
            0xE000..0xFE00 => 0,
            0xFE00..0xFEA0 => self.ppu.rb(a),
            0xFEA0..0xFF00 => 0,
            0xFF00..0xFF80 => match a {
                0xFF00 => self.joypad.rb(a),
                0xFF01 => self.serial_data[0],
                0xFF02 => self.serial_data[1],
                0xFF04..=0xFF07 => self.timer.rb(a),
                0xFF40..=0xFF4B => self.ppu.rb(a),
                0xFF0F => self.intf,
                _ => 0,
            },
            0xFFFF => self.inte,
            _ => self.cart.hram_read(a),
        }
    }

    #[inline(always)]
    pub(crate) fn wb(&mut self, a: u16, v: u8) {
        match a {
            0x0000..0x8000 => self.cart.mbc.write(a, v),
            0x8000..0xA000 => self.ppu.wb(a, v),
            0xA000..0xC000 => self.cart.mbc.write(a, v),
            0xC000..0xE000 => self.cart.wram_write(a, v),
            0xE000..0xFE00 => {}
            0xFE00..0xFEA0 => self.ppu.wb(a, v),
            0xFEA0..0xFF00 => {}
            0xFF46 => self.dma(v),
            0xFF00..0xFF80 => match a {
                0xFF00 => self.joypad.wb(a, v),
                0xFF01 => self.serial_data[0] = v,
                0xFF02 => self.serial_data[1] = v,
                0xFF04..=0xFF07 => self.timer.wb(a, v),
                0xFF40..=0xFF4B => self.ppu.wb(a, v),
                0xFF0F => self.intf = v,
                _ => {}
            },
            0xFFFF => self.inte = v,
            _ => self.cart.hram_write(a, v),
        };
    }

    fn dma(&mut self, value: u8) {
        let base = (value as u16) << 8;

        for i in 0..0xA0 {
            let b = self.rb(base + i);
            self.wb(0xFE00 + i, b);
        }
    }

    #[inline(always)]
    pub(crate) fn rw(&self, address: u16) -> u16 {
        let low = self.rb(address) as u16;
        let high = self.rb(address + 1) as u16;
        low | high << 8
    }

    #[inline(always)]
    pub(crate) fn ww(&mut self, a: u16, v: u16) {
        self.wb(a, (v & 0xFF) as u8);
        self.wb(a + 1, (v >> 8) as u8);
    }

    #[inline(always)]
    pub(crate) fn tick(&mut self, c: u32) {
        self.timer.do_cycle(c);
        self.intf |= self.timer.interrupt;
        self.timer.interrupt = 0;

        self.ppu.do_cycle(c);
        self.intf |= self.ppu.interrupt;
        self.ppu.interrupt = 0;

        self.intf |= self.joypad.interrupt;
        self.joypad.interrupt = 0;
    }
}
