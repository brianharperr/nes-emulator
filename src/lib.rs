pub mod cpu;
pub mod ppu;
pub mod mapper;
pub mod mappers;
pub mod rom;
pub mod memory;
use std::fs;

use cpu::Cpu;
use rom::Rom;
pub enum SystemVersion {
    NTSC,
    PAL,
    Dendy,
    RGB,
    BrazilFamiclone,
    ArgentinaFamiclone
}

pub struct Nes {
    cpu: Cpu
}

impl Nes {
    pub fn new(version: SystemVersion) -> Self {
        Nes {
            cpu: Cpu::new(version)
        }
    }

    pub fn on(&mut self){
        self.cpu.interrupt(cpu::cpu::Interrupt::RESET);
    }

    pub fn off(&mut self){

    }

    pub fn reset(&mut self){
        self.cpu.reset();
    }

    pub fn step(&mut self){
        self.cpu.step();
    }

    pub fn set_rom(&mut self, rom: Rom){
        self.cpu.bus.ppu.rom = rom;
    }

    pub fn set_start(&mut self, addr: u16){
        self.cpu.pc = addr;
    }

    pub fn set_debug_mode(&mut self){
        self.cpu.debug_mode = true;
    }
    
    pub fn poll_frame(&mut self) -> bool{
        let ret = self.cpu.bus.ppu.frame_ready;
        self.cpu.bus.ppu.frame_ready = false;
        ret
    }

    pub fn frame(&mut self) -> [u8;256 * 240 * 3] {
        self.cpu.bus.ppu.frame_buffer
    }

    pub fn dump_ppu(&mut self) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;
    
        let mut file = File::create("nametable_dump.txt")?;
    
        println!("{:X}",self.cpu.bus.ppu.read(0x2060));
        // Write header
        writeln!(file, "NES Nametable Memory Dump")?;
        writeln!(file, "=======================")?;
    
        // Dump all four nametables (0x2000-0x2FFF)
        for nt in 0..4 {
            let base_addr = 0x2000 + (nt * 0x400);
            writeln!(file, "\nNametable {}", nt)?;
            writeln!(file, "-------------")?;
    
            // Print each row of the 32x30 nametable
            for y in 0..30 {
                // Write row number
                write!(file, "{:02X}: ", y)?;
    
                // Write tile values for this row
                for x in 0..32 {
                    let addr = base_addr + y * 32 + x;
                    let tile = self.cpu.bus.ppu.read(addr as u16);
                    write!(file, "{:02X} ", tile)?;
                }
                
                // Add ASCII representation
                write!(file, "| ")?;
                for x in 0..32 {
                    let addr = base_addr + y * 32 + x;
                    let tile = self.cpu.bus.ppu.read(addr as u16);
                    // Convert to ASCII if printable, otherwise use a dot
                    let ch = if tile >= 0x20 && tile < 0x7F {
                        tile as char
                    } else {
                        '.'
                    };
                    write!(file, "{}", ch)?;
                }
                writeln!(file)?;
            }
    
            // Print attribute table for this nametable
            writeln!(file, "\nAttribute Table:")?;
            let attr_base = base_addr + 0x3C0;
            for y in 0..8 {
                write!(file, "    ")?;
                for x in 0..8 {
                    let addr = attr_base + y * 8 + x;
                    let attr = self.cpu.bus.ppu.read(addr as u16);
                    write!(file, "{:02X} ", attr)?;
                }
                writeln!(file)?;
            }
        }

        // Add palette data section
        writeln!(file, "\nPalette Data")?;
        writeln!(file, "============")?;

        // Background palettes (0x3F00-0x3F0F)
        writeln!(file, "\nBackground Palettes:")?;
        for i in 0..4 {
            write!(file, "Palette {}: ", i)?;
            for j in 0..4 {
                let addr = 0x3F00 + i * 4 + j;
                let color = self.cpu.bus.ppu.read(addr as u16);
                write!(file, "{:02X} ", color)?;
            }
            writeln!(file)?;
        }

        // Sprite palettes (0x3F10-0x3F1F)
        writeln!(file, "\nSprite Palettes:")?;
        for i in 0..4 {
            write!(file, "Palette {}: ", i)?;
            for j in 0..4 {
                let addr = 0x3F10 + i * 4 + j;
                let color = self.cpu.bus.ppu.read(addr as u16);
                write!(file, "{:02X} ", color)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    pub fn run(&mut self){
        if self.cpu.debug_mode {
            if fs::metadata("debug.log").is_ok() {
                let _= fs::remove_file("debug.log");  // Delete the file
            }
        }

        if self.cpu.debug_mode {
            if fs::metadata("nt.log").is_ok() {
                let _= fs::remove_file("nt.log");  // Delete the file
            }
        }

        loop{
            self.step();
        }
    }

}