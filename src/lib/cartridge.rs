use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Mbc {
    pub(crate) rom: Vec<u8>,
    rom_bank: u16,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enabled: bool,
    banking_mode: bool,
    secondary_banking: bool,
}

impl Mbc {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        let secondary_banking = data.len() > 0x80000;

        Self {
            rom: data,
            rom_bank: 1,
            ram: vec![0; 0x8000],
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: false,
            secondary_banking,
        }
    }

    pub(crate) fn read(&self, a: u16) -> u8 {
        match a {
            0..0x4000 => self.rom[a as usize],
            0x4000..0x8000 => {
                let rom_addr = (a % 0x4000) + (self.rom_bank * 0x4000);
                self.rom[rom_addr as usize]
            }
            0xA000..0xC000 => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let addr = a - 0xA000;
                let addr = addr + (self.ram_bank as u16 * 0x2000);
                self.ram[addr as usize]
            }
            _ => 0xFF,
        }
    }

    pub(crate) fn write(&mut self, a: u16, v: u8) {
        match a {
            0x0..0x2000 => self.ram_enabled = (v & 0x0F) == 0x0A,
            0x2000..0x4000 => {
                self.rom_bank = v as u16 & 0b11111;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
            0x4000..0x6000 => {
                if self.banking_mode {
                    if self.secondary_banking {
                        self.rom_bank = (self.rom_bank & 0b11111) | (v as u16 & 0b11 << 5);
                    }
                } else {
                    self.ram_bank = v & 0b11;
                }
            }
            0x6000..0x8000 => {
                self.banking_mode = v & 1 == 1;
            }
            0xA000..0xC000 => {
                if self.ram_enabled {
                    let addr = a as usize - 0xA000;
                    let addr = addr + (self.ram_bank as usize * 0x2000);
                    self.ram[addr] = v;
                }
            }
            _ => eprintln!("Write to invalid address."),
        }
    }
}

pub(crate) struct Cartridge {
    pub(crate) hram: [u8; 0x80],
    pub(crate) wram: [u8; 0x8000],
    pub(crate) checksum: u8,
    pub(crate) mbc: Mbc,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            wram: [0x0; 0x8000],
            hram: [0xFF; 0x80],
            checksum: 0,
            mbc: Mbc::default(),
        }
    }
}

impl Cartridge {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from(filename: PathBuf) -> Self {
        let cart_data = std::fs::read(&filename).unwrap();
        let mut cart = Self::default();
        cart.mbc = Mbc::new(cart_data);
        cart.checksum = cart.calculate_checksum();
        cart
    }

    fn calculate_checksum(&self) -> u8 {
        let mut checksum: u8 = 0;

        for address in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(self.mbc.rom[address]);
            checksum = checksum.wrapping_sub(1);
        }

        if self.mbc.rom[0x14D] != checksum {
            eprintln!("Invalid checksum! A real Gameboy doesn't care.");
        }

        checksum
    }

    #[inline(always)]
    pub(crate) fn wram_read(&self, mut address: u16) -> u8 {
        address = address.wrapping_sub(0xC000);

        assert!(
            address < 0x2000,
            "Invalid WRAM address {}",
            address.wrapping_add(0xC000)
        );

        self.wram[address as usize]
    }

    #[inline(always)]
    pub(crate) fn hram_read(&self, mut address: u16) -> u8 {
        address = address.wrapping_sub(0xFF80);

        self.hram[address as usize]
    }

    #[inline(always)]
    pub(crate) fn wram_write(&mut self, mut address: u16, value: u8) {
        address = address.wrapping_sub(0xC000);

        self.wram[address as usize] = value;
    }

    #[inline(always)]
    pub(crate) fn hram_write(&mut self, mut address: u16, value: u8) {
        address = address.wrapping_sub(0xFF80);

        self.hram[address as usize] = value;
    }
}
