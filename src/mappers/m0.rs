use crate::{mapper::Mapper, memory::Memory, rom::header::{RomHeader, HEADER_SIZE}};

pub struct Mapper0 {
	chr_rom: Memory,
    chr_ram: Memory,
    prg_rom: Memory,
    prg_ram: Memory
}

impl Mapper0 {
	pub fn new(header: &RomHeader, data: Vec<u8>) -> Self {
        let prg_rom = Memory::new(data[HEADER_SIZE..HEADER_SIZE + header.prg_rom_size as usize].to_vec());
        let chr_rom = Memory::new(data[HEADER_SIZE + header.prg_rom_size as usize..HEADER_SIZE + header.prg_rom_size as usize + header.chr_rom_size as usize].to_vec());


        let mut chr_ram = Memory::new(vec![0; 0]);
        if header.chr_rom_size ==0 && header.chr_ram_size == 0 {
            chr_ram = Memory::new(vec![0;8 * 1024]);
        }

		Mapper0 {
			chr_rom,
            chr_ram,
            prg_rom,
            prg_ram: Memory::new(vec![0; 8 * 1024]),
		}
	}
}
impl Mapper for Mapper0 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // CHR ROM/RAM (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                if self.chr_rom.capacity() == 0 {
                    self.chr_ram.read(addr)
                }else{
                    self.chr_rom.read(addr)
                }
            }
            
            // PRG RAM (0x6000-0x7FFF)
            0x6000..=0x7FFF => self.prg_ram.read(addr - 0x6000),
            
            // PRG ROM (0x8000-0xFFFF)
            0x8000..=0xFFFF => {
                let mapped_addr = if addr >= 0xC000 && self.prg_rom.capacity() <= 0x4000 {
                    // Mirror for 16KB PRG ROM
                    (addr - 0xC000) % 0x4000
                } else {
                    // 32KB PRG ROM or lower bank access
                    (addr - 0x8000) % self.prg_rom.capacity() as u16
                };
                self.prg_rom.read(mapped_addr)
            },
            
            // Invalid addresses
            _ => {
                debug_assert!(false, "NROM: Invalid read address: ${:04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {

        match addr {
            // CHR RAM writes (if present)
            0x0000..=0x1FFF => {
                // Only write if it's CHR RAM (will be ignored for CHR ROM)
                self.chr_ram.write(addr, data);
            },
            
            // PRG RAM writes
            0x6000..=0x7FFF => {
                self.prg_ram.write(addr - 0x6000, data);
            },
            
            // PRG ROM writes are ignored
            0x8000..=0xFFFF => {
                // Ignore writes to PRG ROM
                debug_assert!(false, "NROM: Attempted write to PRG ROM: ${:04X}", addr);
            },
            
            // Invalid addresses
            _ => {
                debug_assert!(false, "NROM: Invalid write address: ${:04X}", addr);
            }
        }
    }

    fn map(&self, addr: u16) -> u16 {
        match addr {
            // CHR ROM/RAM mapping
            0x0000..=0x1FFF => addr,
            
            // PRG RAM mapping
            0x6000..=0x7FFF => addr - 0x6000,
            
            // PRG ROM mapping
            0x8000..=0xFFFF => {
                if addr >= 0xC000 && self.prg_rom.capacity() <= 0x4000 {
                    // Mirror for 16KB PRG ROM
                    (addr - 0xC000) % 0x4000
                } else {
                    // 32KB PRG ROM or lower bank access
                    (addr - 0x8000) % self.prg_rom.capacity() as u16
                }
            },
            
            // Invalid addresses
            _ => {
                debug_assert!(false, "NROM: Invalid map address: ${:04X}", addr);
                0
            }
        }
    }
}