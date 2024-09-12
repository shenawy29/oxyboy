use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Cartridge {
    pub(crate) rom_data: Vec<u8>,
    pub(crate) hram: [u8; 0x80],
    pub(crate) wram: [u8; 0x8000],
    pub(crate) checksum: u8,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            rom_data: Default::default(),
            hram: [0x0; 0x80],
            wram: [0x0; 0x8000],
            checksum: 0,
        }
    }
}

impl Cartridge {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from(filename: PathBuf) -> Self {
        let cart_data = std::fs::read(&filename).unwrap();

        let mut cart = Self {
            wram: [0x0; 0x8000],
            hram: [0xFF; 0x80],
            rom_data: cart_data,
            checksum: 0,
        };

        cart.checksum = cart.calculate_checksum();

        cart
    }

    fn calculate_checksum(&self) -> u8 {
        let mut checksum: u8 = 0;

        for address in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(self.rom_data[address]);
            checksum = checksum.wrapping_sub(1);
        }

        if self.rom_data[0x14d] != checksum {
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
    pub(crate) fn read(&self, address: u16) -> u8 {
        self.rom_data[address as usize]
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

    // TODO: MBC1 emulation
    pub(crate) fn write(&mut self, _address: u16, _value: u8) {}
}
