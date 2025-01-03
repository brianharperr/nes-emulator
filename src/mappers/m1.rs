use crate::{mapper::Mapper, memory::Memory, rom::header::{RomHeader, HEADER_SIZE}};

pub struct Mapper1 {
    chr_rom: Memory,
    prg_rom: Memory,
    prg_ram: Memory,
    shift_register: u8,
    shift_count: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
    last_write_cycle: u64, // For detecting consecutive writes
}

impl Mapper1 {
    pub fn new(header: &RomHeader, data: Vec<u8>) -> Self {
        let prg_rom_data = data[HEADER_SIZE..HEADER_SIZE + header.prg_rom_size as usize].to_vec();
        let chr_rom_data = data[HEADER_SIZE + header.prg_rom_size as usize..HEADER_SIZE + header.prg_rom_size as usize + header.chr_rom_size as usize].to_vec();

        Mapper1 {
            chr_rom: Memory::new(chr_rom_data),
            prg_rom: Memory::new(prg_rom_data),
            prg_ram: Memory::new(vec![0; 1024 * 8]), // 8KB PRG RAM
            shift_register: 0x10, // Initial state
            shift_count: 0,
            control: 0x0C,       // Initial state: PRG ROM mode 3, CHR ROM mode 0
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            last_write_cycle: 0,
        }
    }

    fn write_register(&mut self, addr: u16, data: u8) {
        // Reset shift register if bit 7 is set
        if data & 0x80 != 0 {
            self.shift_register = 0x10;
            self.shift_count = 0;
            self.control |= 0x0C; // Reset to PRG ROM mode 3
            return;
        }

        // Load shift register
        self.shift_register >>= 1;
        self.shift_register |= (data & 1) << 4;
        self.shift_count += 1;

        // If we've shifted 5 bits, write to the appropriate register
        if self.shift_count == 5 {
            let value = self.shift_register;
            match addr & 0x6000 {
                0x0000 => self.control = value,     // Control register
                0x2000 => self.chr_bank_0 = value,  // CHR bank 0
                0x4000 => self.chr_bank_1 = value,  // CHR bank 1
                0x6000 => self.prg_bank = value,    // PRG bank
                _ => unreachable!()
            }
            
            self.shift_register = 0x10;
            self.shift_count = 0;
        }
    }
}

impl Mapper for Mapper1 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // CHR ROM (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                let chr_mode = (self.control >> 4) & 1;
                let bank = if chr_mode == 0 {
                    // 8KB mode
                    let bank = (self.chr_bank_0 & 0x1E) as u16;
                    (addr + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                } else {
                    // 4KB mode
                    if addr < 0x1000 {
                        let bank = self.chr_bank_0 as u16;
                        (addr + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                    } else {
                        let bank = self.chr_bank_1 as u16;
                        ((addr - 0x1000) + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                    }
                };
                self.chr_rom.read(bank)
            },

            // PRG RAM (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                self.prg_ram.read(addr - 0x6000)
            },

            // PRG ROM (0x8000-0xFFFF)
            0x8000..=0xFFFF => {
                let prg_mode = (self.control >> 2) & 0x3;
                let mapped_addr = match prg_mode {
                    0 | 1 => {
                        // 32KB mode
                        let bank = (self.prg_bank & 0x0E) as u32;
                        ((addr - 0x8000) as u32 + (bank * 0x4000)) as u16
                    },
                    2 => {
                        // Fix first bank, switch second
                        if addr < 0xC000 {
                            addr - 0x8000
                        } else {
                            ((addr - 0xC000) as u32 + (self.prg_bank as u32 * 0x4000)) as u16
                        }
                    },
                    3 => {
                        // Fix last bank, switch first
                        if addr >= 0xC000 {
                            (addr - 0xC000) + (self.prg_rom.capacity() as u16 - 0x4000)
                        } else {
                            ((addr - 0x8000) as u32 + (self.prg_bank as u32 * 0x4000)) as u16
                        }
                    },
                    _ => unreachable!()
                };
                self.prg_rom.read(mapped_addr % self.prg_rom.capacity() as u16)
            },

            _ => 0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // CHR ROM/RAM (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                self.chr_rom.write(addr, data); // Will be ignored if ROM
            },

            // PRG RAM (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                self.prg_ram.write(addr - 0x6000, data);
            },

            // Register writes (0x8000-0xFFFF)
            0x8000..=0xFFFF => {
                self.write_register(addr, data);
            },

            _ => {}
        }
    }

    fn map(&self, addr: u16) -> u16 {
        match addr {
            // CHR ROM/RAM mapping
            0x0000..=0x1FFF => {
                let chr_mode = (self.control >> 4) & 1;
                if chr_mode == 0 {
                    // 8KB mode
                    let bank = (self.chr_bank_0 & 0x1E) as u16;
                    (addr + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                } else {
                    // 4KB mode
                    if addr < 0x1000 {
                        let bank = self.chr_bank_0 as u16;
                        (addr + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                    } else {
                        let bank = self.chr_bank_1 as u16;
                        ((addr - 0x1000) + (bank * 0x1000)) % self.chr_rom.capacity() as u16
                    }
                }
            },

            // PRG RAM mapping
            0x6000..=0x7FFF => addr - 0x6000,

            // PRG ROM mapping
            0x8000..=0xFFFF => {
                let prg_mode = (self.control >> 2) & 0x3;
                match prg_mode {
                    0 | 1 => {
                        // 32KB mode
                        let bank = (self.prg_bank & 0x0E) as u32;
                        ((addr - 0x8000) as u32 + (bank * 0x4000)) as u16
                    },
                    2 => {
                        // Fix first bank, switch second
                        if addr < 0xC000 {
                            addr - 0x8000
                        } else {
                            ((addr - 0xC000) as u32 + (self.prg_bank as u32 * 0x4000)) as u16
                        }
                    },
                    3 => {
                        // Fix last bank, switch first
                        if addr >= 0xC000 {
                            (addr - 0xC000) + (self.prg_rom.capacity() as u16 - 0x4000)
                        } else {
                            ((addr - 0x8000) as u32 + (self.prg_bank as u32 * 0x4000)) as u16
                        }
                    },
                    _ => unreachable!()
                }
            },

            _ => addr
        }
    }
}