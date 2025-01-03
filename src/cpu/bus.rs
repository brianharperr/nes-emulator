use crate::{memory::Memory, ppu::Ppu, ppu2::Ppu2};

const CPU_RAM_SIZE: usize = 0x800; //2KB

pub struct Bus {
    ram: Memory,
    pub ppu: Ppu,

    pub cycles: u64,
    pub reset: bool,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Memory::new(vec![0; CPU_RAM_SIZE]),
            ppu: Ppu::new(),
            cycles: 0,
            reset: false,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => {
                self.ram.read(addr & 0x7FF)
            }
            0x2000..0x4000 => {
                let m_addr = addr & 0x2007;
                match m_addr {
                    0x2002 => self.ppu.read_status(),
                    0x2004 => self.ppu.read_oam(),
                    0x2007 => self.ppu.read_data(),
                    _ => self.ppu.open_bus
                }
            }
            0x4000..0x4020 => { //APU / I/O
                0
            }
            0x4020..=0xFFFF => {
                self.ppu.rom.mapper.read(addr)
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr == 0x2017 {
            println!("HITTTTT");
        }
        match addr {
            0x0000..0x2000 => {
                self.ram.write(addr & 0x7FF, data);
            }
            0x2000..0x4000 => {
                let m_addr = addr & 0x2007;
                match m_addr {
                    0x2000 => if !self.ignore_ppu_writes() { self.ppu.write_ctrl(data) },
                    0x2001 => if !self.ignore_ppu_writes() { self.ppu.write_mask(data) },
                    0x2003 => self.ppu.write_oamaddr(data),
                    0x2004 => self.ppu.write_oamdata(data),
                    0x2005 => if !self.ignore_ppu_writes() { self.ppu.write_scroll(data) },
                    0x2006 => if !self.ignore_ppu_writes() { self.ppu.write_addr(data) },
                    0x2007 => self.ppu.write_data(data),
                    _ => {}
                }
                self.ppu.open_bus = data;
            }
            0x4000..0x4020 => { //APU / I/O
                
            }
            0x4020..=0xFFFF => {
                self.ppu.rom.mapper.write(addr, data);
                //RAM write
            }
        }
    }

    //Wil
    fn ignore_ppu_writes(&self) -> bool {
        self.reset && self.cycles < 29658
    }
}