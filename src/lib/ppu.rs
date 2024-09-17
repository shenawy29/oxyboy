use std::cmp::Ordering;

const VRAM_LEN: usize = 0x4000;
const VOAM_LEN: usize = 0xA0;

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

#[derive(PartialEq, Clone, Copy)]
enum Priority {
    Color0,
    Normal,
}

bitflags::bitflags! {
    pub struct Lcds: u8 {
        const LYC_INTE      = 0b0100_0000;
        const M2_INTE       = 0b0010_0000;
        const M1_INTE       = 0b0001_0000;
        const M0_INTE       = 0b0000_1000;
        const COINCIDENCE   = 0b0000_0100;
        const MODE          = 0b0000_0011;
    }

    pub struct Lcdc: u8 {
        const LCD_ON        = 0b1000_0000;
        const WIN_TILEMAP   = 0b0100_0000;
        const WIN_ON        = 0b0010_0000;
        const TILE_DATA     = 0b0001_0000;
        const BG_TILEMAP    = 0b0000_1000;
        const SPRITE_SIZE   = 0b0000_0100;
        const SPRITE_ON     = 0b0000_0010;
        const BG_WIN_ENABLE = 0b0000_0001;
    }
}

pub(crate) struct Ppu {
    lcds: Lcds,
    lcdc: Lcdc,
    ly: u8,
    lyc: u8,
    scy: u8,
    scx: u8,
    winy: u8,
    winx: u8,

    clock: u32,

    wy_trigger: bool,
    wy_pos: i32,

    palbr: u8,
    pal0r: u8,
    pal1r: u8,

    palb: [u8; 4],
    pal0: [u8; 4],
    pal1: [u8; 4],

    vram: [u8; VRAM_LEN],
    voam: [u8; VOAM_LEN],

    bgprio: [Priority; SCREEN_W],

    pub buffer: [u8; 69120],
    pub updated: bool,
    pub interrupt: u8,
}

impl Ppu {
    pub(crate) fn new() -> Ppu {
        Ppu {
            lcds: Lcds::empty(),
            lcdc: Lcdc::empty(),

            clock: 456,

            ly: 0,
            lyc: 0,

            scy: 0,
            scx: 0,

            winy: 0,
            winx: 0,

            wy_trigger: false,
            wy_pos: -1,

            palbr: 0,
            pal0r: 0,
            pal1r: 1,
            palb: [0; 4],
            pal0: [0; 4],
            pal1: [0; 4],

            vram: [0; VRAM_LEN],
            voam: [0; VOAM_LEN],
            bgprio: [Priority::Normal; SCREEN_W],
            buffer: [0; SCREEN_W * SCREEN_H * 3],
            updated: false,
            interrupt: 0,
        }
    }

    pub(crate) fn do_cycle(&mut self, ticks: u32) {
        // This check makes some games not work for some reason
        // if !self.lcdc(Lcdc::LCD_ON) {
        //     return;
        // }

        let overflow: bool;
        (self.clock, overflow) = self.clock.overflowing_sub(ticks);

        if overflow {
            self.clock = 456;
            self.ly = (self.ly + 1) % 154;
            self.coincidence();

            if self.ly >= 144 && self.lcds.bits() & 0b11 != 1 {
                self.change_mode(1);
            }
        }

        #[rustfmt::skip]
        if self.ly < 144 {
            match self.clock {
                0..81   => (self.mode() != 0).then(|| self.change_mode(0)),
                81..253 => (self.mode() != 3).then(|| self.change_mode(3)),
                _       => (self.mode() != 2).then(|| self.change_mode(2)),
            };
        };
    }

    fn coincidence(&mut self) {
        if self.lcds(Lcds::LYC_INTE) && self.ly == self.lyc {
            self.interrupt |= 0x02;
        }
    }

    fn setmode(&mut self, mode: u8) {
        self.lcds &= Lcds::from_bits_retain(!0b11);
        self.lcds |= Lcds::from_bits_retain(mode);
    }

    fn mode(&self) -> u8 {
        self.lcds.bits() & 0b11
    }

    fn change_mode(&mut self, mode: u8) {
        self.setmode(mode);

        let lcd_interrupt = match self.mode() {
            0 => {
                self.renderscan();
                self.lcds(Lcds::M0_INTE)
            }
            1 => {
                self.wy_trigger = false;
                self.interrupt |= 0x01;
                self.updated = true;
                self.lcds(Lcds::M1_INTE)
            }
            2 => self.lcds(Lcds::M2_INTE),
            3 => {
                if !self.wy_trigger && self.ly == self.winy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }

                false
            }

            _ => unreachable!(),
        };

        if lcd_interrupt {
            self.interrupt |= 0x02;
        }
    }

    pub(crate) fn rb(&self, a: u16) -> u8 {
        match a {
            0x8000..0xA000 => self.vram[a as usize - 0x8000],
            0xFE00..0xFEA0 => self.voam[a as usize - 0xFE00],
            0xFF40 => self.lcdc.bits(),
            0xFF41 => self.lcds.bits(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,

            0xFF47 => self.palbr,
            0xFF48 => self.pal0r,
            0xFF49 => self.pal1r,
            0xFF4A => self.winy,
            0xFF4B => self.winx,
            _ => 0xFF,
        }
    }

    fn lcds(&self, bit: Lcds) -> bool {
        self.lcds.contains(bit)
    }

    fn lcdc(&self, bit: Lcdc) -> bool {
        self.lcdc.contains(bit)
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        match a {
            0x8000..0xA000 => self.vram[a as usize - 0x8000] = v,
            0xFE00..0xFEA0 => self.voam[a as usize - 0xFE00] = v,
            0xFF40 => {
                let orig_lcd_on = self.lcdc(Lcdc::LCD_ON);

                self.lcdc = Lcdc::from_bits_retain(v);

                if orig_lcd_on && !self.lcdc(Lcdc::LCD_ON) {
                    self.clock = 456;
                    self.ly = 0;
                    self.setmode(0);
                    self.wy_trigger = false;
                    self.clear_screen();
                }

                if !orig_lcd_on && self.lcdc(Lcdc::LCD_ON) {
                    self.change_mode(2);
                    self.clock = 452;
                }
            }
            0xFF41 => self.lcds = Lcds::from_bits_retain(v),
            0xFF42 => self.scy = v,
            0xFF43 => self.scx = v,

            0xFF45 => {
                self.lyc = v;
                self.coincidence();
            }

            0xFF47 => {
                self.palbr = v;
                self.update_pal();
            }
            0xFF48 => {
                self.pal0r = v;
                self.update_pal();
            }
            0xFF49 => {
                self.pal1r = v;
                self.update_pal();
            }
            0xFF4A => self.winy = v,
            0xFF4B => self.winx = v,
            _ => unreachable!(),
        }
    }

    fn clear_screen(&mut self) {
        self.buffer.fill(0xFF);
        self.updated = true;
    }

    fn update_pal(&mut self) {
        for i in 0..4 {
            self.palb[i] = Ppu::get_monochrome_pal_val(self.palbr, i);
            self.pal0[i] = Ppu::get_monochrome_pal_val(self.pal0r, i);
            self.pal1[i] = Ppu::get_monochrome_pal_val(self.pal1r, i);
        }
    }

    fn renderscan(&mut self) {
        for x in 0..SCREEN_W {
            self.setcolor(x, 255);
            self.bgprio[x] = Priority::Normal;
        }

        self.draw_bg();
        self.draw_window();
        self.draw_sprites();
    }

    fn get_monochrome_pal_val(value: u8, index: usize) -> u8 {
        match (value >> (2 * index)) & 0x03 {
            0 => 0xFF,
            1 => 0xC0,
            2 => 0x60,
            _ => 0,
        }
    }

    fn setcolor(&mut self, x: usize, color: u8) {
        self.buffer[self.ly as usize * SCREEN_W * 3 + x * 3] = color;
        self.buffer[self.ly as usize * SCREEN_W * 3 + x * 3 + 1] = color;
        self.buffer[self.ly as usize * SCREEN_W * 3 + x * 3 + 2] = color;
    }

    fn draw_bg(&mut self) {
        let drawbg = self.lcdc(Lcdc::BG_WIN_ENABLE);

        if !drawbg {
            return;
        }

        let bgy = self.scy.wrapping_add(self.ly);

        let bgtiley = bgy as u16 / 8;

        for x in 0..SCREEN_W {
            let bgx = self.scx as u16 + x as u16;

            let bgtilex = (bgx / 8) & 31;

            let tilemapbase = if self.lcdc(Lcdc::BG_TILEMAP) {
                0x9C00
            } else {
                0x9800
            };

            let tilex = bgtilex;
            let tiley = bgtiley;
            let pixely = bgy & 0b111;
            let pixelx = bgx & 0b111;

            let tile_num = self.rb(tilemapbase + tiley * 32 + tilex);

            let tilebase = if self.lcdc(Lcdc::TILE_DATA) {
                0x8000
            } else {
                0x8800
            };

            let tileaddress = if tilebase == 0x8000 {
                0x8000u16 + tile_num as u16 * 16
            } else {
                0x8800u16 + ((tile_num as i8 as i16 + 128) * 16) as u16
            };

            let addr = tileaddress + (pixely as u16 * 2);

            let b1 = self.rb(addr);
            let b2 = self.rb(addr + 1);

            let bit = 7 - pixelx;

            let lo = (b1 >> bit) & 1;
            let hi = (b2 >> bit) & 1;

            let colnr = (hi << 1) | lo;

            self.bgprio[x] = if colnr == 0 {
                Priority::Color0
            } else {
                Priority::Normal
            };

            let color = self.palb[colnr as usize];
            self.setcolor(x, color);
        }
    }

    fn draw_window(&mut self) {
        let wx_trigger = self.winx <= 166;

        let winy = if self.lcdc(Lcdc::WIN_ON) && self.wy_trigger && wx_trigger {
            self.wy_pos += 1;
            self.wy_pos
        } else {
            -1
        };

        if winy < 0 {
            return;
        }

        let wintiley = (winy as u16 / 8) & 31;

        for x in 0..SCREEN_W {
            let winx = -((self.winx as i32) - 7) + (x as i32);

            if !(winy >= 0 && winx >= 0) {
                continue;
            }

            let tilemapbase = if self.lcdc(Lcdc::WIN_TILEMAP) {
                0x9C00
            } else {
                0x9800
            };

            let tiley = wintiley;
            let tilex = winx as u16 / 8;
            let pixely = winy as u16 & 0x07;
            let pixelx = winx as u8 & 0x07;

            let tile_num = self.rb(tilemapbase + tiley * 32 + tilex);

            let tilebase = if self.lcdc(Lcdc::TILE_DATA) {
                0x8000
            } else {
                0x8800
            };

            let tileaddress = if tilebase == 0x8000 {
                0x8000u16 + tile_num as u16 * 16
            } else {
                0x8800u16 + ((tile_num as i8 as i16 + 128) * 16) as u16
            };

            let addr = tileaddress + (pixely * 2);

            let b1 = self.rb(addr);
            let b2 = self.rb(addr + 1);

            let bit = 7 - pixelx;

            let lo = (b1 >> bit) & 1;
            let hi = (b2 >> bit) & 1;

            let colnr = (hi << 1) | lo;

            self.bgprio[x] = if colnr == 0 {
                Priority::Color0
            } else {
                Priority::Normal
            };

            let color = self.palb[colnr as usize];
            self.setcolor(x, color);
        }
    }

    fn draw_sprites(&mut self) {
        if !self.lcdc(Lcdc::SPRITE_ON) {
            return;
        }

        let line = self.ly as i32;

        let sprite_size = if self.lcdc(Lcdc::SPRITE_SIZE) { 16 } else { 8 };

        let mut sprites_to_draw = [(0, 0, 0); 10];

        let mut sidx = 0;

        for i in 0..40 {
            let spriteaddr = 0xFE00 + (i as u16) * 4;

            let spritey = self.rb(spriteaddr) as u16 as i32 - 16;

            if line < spritey || line >= spritey + sprite_size {
                continue;
            }

            let spritex = self.rb(spriteaddr + 1) as u16 as i32 - 8;

            sprites_to_draw[sidx] = (spritex, spritey, i);

            sidx += 1;

            if sidx >= 10 {
                break;
            }
        }

        sprites_to_draw[..sidx].sort_unstable_by(dmg_sprite_order);

        for &(spritex, spritey, i) in &sprites_to_draw[..sidx] {
            if spritex < -7 || spritex >= (SCREEN_W as i32) {
                continue;
            }

            let spriteaddr = 0xFE00 + (i as u16) * 4;
            let tilenum =
                (self.rb(spriteaddr + 2) & (if sprite_size == 16 { 0xFE } else { 0xFF })) as u16;
            let flags = self.rb(spriteaddr + 3) as usize;
            let usepal1: bool = flags & (1 << 4) != 0;
            let xflip: bool = flags & (1 << 5) != 0;
            let yflip: bool = flags & (1 << 6) != 0;
            let belowbg: bool = flags & (1 << 7) != 0;

            let tiley: u16 = if yflip {
                (sprite_size - 1 - (line - spritey)) as u16
            } else {
                (line - spritey) as u16
            };

            let tileaddress = 0x8000 + tilenum * 16 + tiley * 2;

            let (b1, b2) = (self.rb(tileaddress), self.rb(tileaddress + 1));

            'xloop: for x in 0..8 {
                if spritex + x < 0 || spritex + x >= (SCREEN_W as i32) {
                    continue;
                }

                let xbit = 1 << (if xflip { x } else { 7 - x } as u32);

                let colnr =
                    (if b1 & xbit != 0 { 1 } else { 0 }) | (if b2 & xbit != 0 { 2 } else { 0 });
                if colnr == 0 {
                    continue;
                }

                if belowbg && self.bgprio[(spritex + x) as usize] != Priority::Color0 {
                    continue 'xloop;
                }

                let color = if usepal1 {
                    self.pal1[colnr]
                } else {
                    self.pal0[colnr]
                };

                self.setcolor((spritex + x) as usize, color);
            }
        }
    }
}

fn dmg_sprite_order(a: &(i32, i32, u8), b: &(i32, i32, u8)) -> Ordering {
    if a.0 != b.0 {
        return b.0.cmp(&a.0);
    }

    b.2.cmp(&a.2)
}
