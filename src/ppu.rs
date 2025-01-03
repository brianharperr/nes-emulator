use core::panic;
use std::{fs::OpenOptions, io::{self, Write}, iter::Scan};

use crate::{memory::Memory, rom::{header::{Mirroring, HEADER_SIZE}, Rom}};

pub static PALETTE: [u8; 192] = [
    124, 124, 124, 
    0, 0, 252, 
    0, 0, 188, 
    68, 40, 188, 
    148, 0, 132, 
    168, 0, 32, 
    168, 16, 0, 
    136, 20, 0, 
    80, 48, 0, 
    0, 120, 0, 
    0, 104, 0, 
    0, 88, 0, 
    0, 64, 88, 
    0, 0, 0, 
    0, 0, 0, 
    0, 0, 0, 
    188, 188, 188, 
    0, 120, 
    248, 0, 88, 248, 104, 68, 252, 216, 0, 204, 228, 0, 88, 248, 56, 0, 228, 92, 16,
    172, 124, 0, 0, 184, 0, 0, 168, 0, 0, 168, 68, 0, 136, 136, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248,
    248, 248, 60, 188, 252, 104, 136, 252, 152, 120, 248, 248, 120, 248, 248, 88, 152, 248, 120,
    88, 252, 160, 68, 248, 184, 0, 184, 248, 24, 88, 216, 84, 88, 248, 152, 0, 232, 216, 120, 120,
    120, 0, 0, 0, 0, 0, 0, 252, 252, 252, 164, 228, 252, 184, 184, 248, 216, 184, 248, 248, 184,
    248, 248, 164, 192, 240, 208, 176, 252, 224, 168, 248, 216, 120, 216, 248, 120, 184, 248, 184,
    184, 248, 216, 0, 252, 252, 248, 216, 248, 0, 0, 0, 0, 0, 0,
];

const PPU_VRAM_SIZE: usize = 0x800; //2KB
const NUM_SCANLINES: usize = 262;
const CYCLERS_PER_SCANLINE: usize = 341;

#[derive(PartialEq)]
pub enum Scanline{
    PreRender,
    Visible,
    PostRender,
    VBlank
}

impl Scanline {
    pub fn from(scanline: usize) -> Scanline {
        match scanline {
            0..=239 => Scanline::Visible,
            240 => Scanline::PostRender,
            241..=260 => Scanline::VBlank,
            261 => Scanline::PreRender,
            _ => panic!("Scanline out of bounds!")
        }
    }
}
pub struct Ppu {

    ctrl: u8,
    mask: u8,
    status: u8,
    oamaddr: u8,

    v: u16,
    t: u16,
    x: u8,
    w: bool,

    odd_frame: bool,

    vram_buffer: u8,
    pub open_bus: u8,
    vram: Memory,
    pub rom: Rom,
    palette: [u8; 32],
    oam: [u8; 256],
    secondary_oam: [u8; 32],

    pub trigger_nmi: bool,

    pub cycle: usize,
    pub scanline: usize,

    pub frame_ready: bool,
    pub frame_buffer: [u8; 256 * 240 * 3],

    nt_byte: u8,
    at_latch_lo: u8,
    at_latch_hi: u8,
    pt_latch_lo: u8,
    pt_latch_hi: u8,

    at_shifter_lo: u8,
    at_shifter_hi: u8,
    pt_shifter_lo: u16,
    pt_shifter_hi: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {

            ctrl: 0,
            mask: 0,
            status: 0,
            oamaddr: 0,

            v: 0,
            t: 0,
            x: 0,
            w: false,

            odd_frame: false,

            vram_buffer: 0,
            open_bus: 0,
            vram: Memory::new(vec![0; PPU_VRAM_SIZE]),
            rom: Rom::new(vec![0; HEADER_SIZE]),
            palette: [0; 32],
            oam: [0; 256],
            secondary_oam: [0; 32],
            trigger_nmi: false,

            cycle: 0,
            scanline: 0,

            frame_buffer: [0; 256 * 240 * 3],
            frame_ready: false,

            nt_byte: 0,
            at_latch_lo: 0,
            at_latch_hi: 0,
            pt_latch_lo: 0,
            pt_latch_hi: 0,

            at_shifter_lo: 0,
            at_shifter_hi: 0,
            pt_shifter_lo: 0,
            pt_shifter_hi: 0,
        }
    }

    pub fn step(&mut self){
        //println!("{}, {}", self.scanline, self.cycle);
        let render = self.is_rendering_enabled();
        match Scanline::from(self.scanline) {
            Scanline::PreRender => {

                if self.cycle == 0 {
                    self.odd_frame = !self.odd_frame;
                }

                if render {
                self.load_pixel();
                self.load_shift_registers();
                }
            }
            Scanline::Visible => {

                if render {
                    self.load_pixel();
                    self.load_shift_registers();
                }
                //SPIRTE EVAULATION FOR NEXT SCANLINE
            }
            Scanline::PostRender => {}
            Scanline::VBlank => {
                if self.scanline == 241 && self.cycle == 1  {
                    self.status |= 0x80;
                    if self.ctrl & 0x80 != 0 {
                        self.trigger_nmi = true;
                        self.frame_ready = true;
                    }
                }
            }
        }

        let skip_cycle: bool = self.scanline == 261 && self.cycle == 339 && self.odd_frame && render;
        if skip_cycle {
            self.cycle = 0;
            self.scanline = 0;
        } else {
            self.cycle += 1;
            if self.cycle >= CYCLERS_PER_SCANLINE {
                self.cycle = 0;
                self.scanline = (self.scanline + 1) % NUM_SCANLINES;
            }
        }

        self.update_scroll();
    }


    fn update_scroll(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
    
        // During rendering scanlines (including pre-render)
        if self.scanline < 240 || self.scanline == 261 {
            // Horizontal position updates every 8 cycles during fetch periods
            if self.cycle > 0 && self.cycle <= 256 {
                if self.cycle % 8 == 0 {
                    self.increment_h();
                }
            } else if self.cycle >= 321 && self.cycle <= 336 {
                if self.cycle % 8 == 0 {
                    self.increment_h();
                }
            }
    
            // Vertical position update at end of scanline
            if self.cycle == 256 {
                self.increment_v();
            }
    
            // Reset horizontal bits after visible part
            if self.cycle == 257 {
                self.copy_h();
            }
    
            // Reset vertical bits during pre-render scanline
            if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
                self.copy_v();
            }
        }
    }

    fn load_shift_registers(&mut self) {
        match self.cycle {
            1..=256 | 321..=336 => {
                self.pt_shifter_lo <<= 1;
                self.pt_shifter_hi <<= 1;
                self.at_shifter_lo <<= 1;
                self.at_shifter_hi <<= 1;

                match (self.cycle - 1) % 8 {
                    0 => { //Correct
                        self.nt_byte = self.get_nt();
                    }
                    2 => { //Correct
                        let at_byte = self.get_attribute();
                        let shift = ((self.v >> 4) & 4) | (self.v & 2);
                        let attr_bits = (at_byte >> shift) & 0x3;
                        self.at_latch_lo = attr_bits & 1;
                        self.at_latch_hi = (attr_bits >> 1) & 1;
                    }
                    4 => { //Correct
                        let addr = self.bg_pattern_table_address() + (self.nt_byte as u16 * 16) + self.fine_y();
                        self.pt_latch_lo = self.read(addr);
                    }
                    6 => { //Correct
                        let addr = self.bg_pattern_table_address() + (self.nt_byte as u16 * 16) + self.fine_y();
                        self.pt_latch_hi = self.read(addr + 8);
                    }
                    7 => {
                        self.pt_shifter_lo = (self.pt_shifter_lo & 0xFF00) | self.pt_latch_lo as u16;
                        self.pt_shifter_hi = (self.pt_shifter_hi & 0xFF00) | self.pt_latch_hi as u16;

                        if self.at_latch_lo != 0 {
                            self.at_shifter_lo |= 0xFF;
                        }
                        if self.at_latch_hi != 0 {
                            self.at_shifter_hi |= 0xFF;
                        }
                    }
                    _ => {}
                }
            }
            257..=320 => {
            }
            337 => {
                self.nt_byte = self.get_nt();
            }
            339 => {
                self.nt_byte = self.get_nt();
            }
            _ => {}

        }
    }

    fn load_pixel(&mut self) {
        // Only render during visible scanlines and cycles
        if self.scanline >= 240 || self.cycle > 256 || self.cycle == 0 {
            return;
        }
    

        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;
    
        // println!("{}", self.is_bg_rendering_enabled());
        if self.is_bg_rendering_enabled() {
            // Skip the leftmost 8 pixels if background clipping is enabled
            if self.cycle >= 8 || self.is_leftmost_bg_rendering_enabled() {
                // Get the bit position from the shift registers
                let fine_x = self.x & 0x7;
                let shift = (15 - fine_x) as u16;
                bg_pixel = ((((self.pt_shifter_hi >> shift) & 1) << 1) | ((self.pt_shifter_lo >> shift) & 1)) as u8;

                // Extract palette bits from attribute shifters
                // Attribute shifters are synchronized with pattern table shifters
                bg_palette = (((self.at_shifter_hi >> (7 - fine_x)) & 1) << 1) | ((self.at_shifter_lo >> (7 - fine_x)) & 1);
            }
        }
    
        let mut sprite_pixel = 0u8;
        let mut sprite_palette = 0u8;
        let mut sprite_priority = false;
        
        // if self.is_sprite_rendering_enabled() {
        //     // Skip the leftmost 8 pixels if sprite clipping is enabled
        //     if self.cycle >= 8 || self.is_leftmost_sprite_rendering_enabled() {
        //         for i in 0..8 {
        //             if self.sprite_scanline[i].x == 0 {
        //                 let sprite_pattern = ((self.sprite_shifter_pattern_hi[i] & 0x80) >> 6) |
        //                                 ((self.sprite_shifter_pattern_lo[i] & 0x80) >> 7);
                        
        //                 if sprite_pattern != 0 {
        //                     sprite_pixel = sprite_pattern;
        //                     sprite_palette = (self.sprite_scanline[i].attributes & 0x3) + 4;
        //                     sprite_priority = (self.sprite_scanline[i].attributes & 0x20) == 0;
        //                     break;
        //                 }
        //             }
        //         }
        //     }
        // }

        let palette_idx = if bg_pixel == 0 { 0 } else { (bg_palette << 2) | bg_pixel };
        let color = (self.palette[palette_idx as usize] & 0x3F) as usize;
        let x = self.cycle - 1;
        let y = self.scanline;
        let idx = (y * 256 + x) * 3;

        // Update frame buffer with bounds checking
        if idx + 2 < self.frame_buffer.len() {
            self.frame_buffer[idx] = PALETTE[color * 3];
            self.frame_buffer[idx + 1] = PALETTE[(color * 3) + 1];
            self.frame_buffer[idx + 2] = PALETTE[(color * 3) + 2];
        }

    }

    fn get_nt(&mut self) -> u8{
        let addr = 0x2000 | (self.v & 0x0FFF);
        let data = self.read(addr);
        data
    }

    fn get_attribute(&mut self) -> u8{
        let addr = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let data = self.read(addr);
        data
    }


    pub fn read(&mut self, addr: u16) -> u8 {
        let mut m_addr = addr & 0x3FFF; //PPU has 14-bit memory space

        match m_addr {
            0x0000..0x2000 => {
                self.rom.mapper.read(m_addr)
            }
            0x2000..0x3F00 => {
                //NAMETABLES
                let v_addr = self.map_vram_addr(m_addr);
                self.vram.read(v_addr)
            }
            0x3F00..0x4000 => {
                //PALETTES
                m_addr = (m_addr - 0x3F00) % 0x20;
                self.palette[m_addr as usize]
            }
            _ => self.open_bus
        }
    }

    fn append_to_file(&self, filename: &str, content: &str) -> io::Result<()> {
        
        let mut file = OpenOptions::new()
        .create(true)  // Create the file if it doesn't exist
        .append(true)  // Append to the file
        .open(filename)?;
    
        // Write the content to the file
        file.write_all(content.as_bytes())?;
        Ok(())
    }
    
    fn write(&mut self, addr: u16, data: u8){
        let mut m_addr = addr & 0x3FFF; //PPU has 14-bit memory space
        self.open_bus = data; 

        match m_addr {
            0x0000..0x2000 => {
                //PATTERN TABLE
            }
            0x2000..0x3000 => {
                //NAMETABLES
                let mirr_addr = self.map_vram_addr(m_addr);
                let output_str = format!(
                    "{:02X} => {:04X}\n",
                    data, mirr_addr
                );
                match self.append_to_file("nt.log", &output_str) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error writing to file: {}", e),
                }
                self.vram.write(mirr_addr, data);
            }
            0x3000..0x3F00 => {
                self.write(addr - 0x1000, data);
            }
            0x3F00..0x4000 => {
                //PALETTES
                m_addr = (m_addr - 0x3F00) % 0x20;
                
                // Always write to the palette address
                self.palette[m_addr as usize] = data;
                // Handle mirroring of the background color ($3F00, $3F04, $3F08, $3F0C)
                if m_addr % 4 == 0 {
                    self.palette[m_addr as usize ^ 0x10] = data;
                }
            }
            _ => panic!("PPU write outside of addressable memory: {:X} => 0x{:X}", data, addr)
        }
    }

    fn map_vram_addr(&mut self, addr: u16) -> u16 {
        // First, get the address within the nametable range (0x2000-0x3EFF)
        let addr = addr & 0x3FFF;
    
        // If not in nametable range (0x2000-0x3EFF), something is wrong
        if addr < 0x2000 {
            panic!("map_vram_addr called with non-nametable address");
        }
        
        // Mirror 0x3000-0x3EFF down to 0x2000-0x2EFF
        let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr };
        
        // Get the nametable number (0-3) and offset within the table
        let nametable = ((addr - 0x2000) >> 10) & 0x3;
        let offset = (addr - 0x2000) & 0x3FF;  // Offset within nametable (0-1023)
        
        // Map the nametable number based on mirroring mode
        let mapped_table = match self.rom.header.mirroring {
            Mirroring::Horizontal => {
                if nametable < 2 { 0 } else { 1 }
            }
            Mirroring::Vertical => {
                nametable & 0x1
            }
            Mirroring::SingleScreen => {
                0
            }
            Mirroring::FourScreen => {
                nametable
            }
        };
    
        // Return index into our 2KB VRAM
        (mapped_table * 0x400) + offset
    }
    
    //Register Functions
    pub fn read_status(&mut self) -> u8{
        let data = self.status;
        self.status &= !0x80; //Clear VBlank flag
        self.w = false;
        self.open_bus = (data & 0xE0) | (self.open_bus & 0x1F);
        data
    }

    //Might not work check addresses
    pub fn read_oam(&self) -> u8{
        self.oam[self.oamaddr as usize]
    }

    pub fn read_data(&mut self) -> u8{
        //If 0x3F00-0x3FFF 
        //return palette data immediately
        //set buffer from normal vram

        //else
        //return buffer
        //set buffer from normal vram

        let data = if (self.v & 0x3FFF) >= 0x3F00 {
            // TODO: Set vram buffer to data thats at mirrorned nt address using palette addr self.vram_buffer = self.vram;
            self.read(self.v)
        }else{
            let previous_buffer = self.vram_buffer;
            self.vram_buffer = self.read(self.v);
            previous_buffer
        };

        self.increment_vram_addr();
        data
    }

    pub fn write_ctrl(&mut self, data: u8){
        //Ignore writes after power/reset until first pre-render scanline
        let old_ctrl = self.ctrl;
        self.ctrl = data;
        if old_ctrl & 0x80 == 0 && self.ctrl & 0x80 == 1 && self.status & 0x80 == 1 {
            self.trigger_nmi = true;
        }

        self.t &= 0xF3FF;
        self.t |= (data as u16 & 0x3) << 10;
    }

    pub fn write_mask(&mut self, data: u8){
        //Ignore writes after power/reset until first pre-render scanline
        self.mask = data;
    }

    pub fn write_oamaddr(&mut self, data: u8){
        self.oamaddr = data;
    }

    //Maybe wrong, check addresses, shouldnt write to anyway
    pub fn write_oamdata(&mut self, data: u8){
        self.oam[self.oamaddr as usize] = data;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    pub fn write_scroll(&mut self, data: u8){
        if !self.w {
            self.x = data & 0x07;
            self.t = (self.t & 0xFFE0) | ((data >> 3) as u16);
        }else{
            self.t = (self.t & 0x8C1F) | ((data as u16 & 0xF8) << 2) | ((data as u16 & 0x07) << 12);
        }

        self.w = !self.w;
    }

    pub fn write_addr(&mut self, data: u8){
        if !self.w {
            self.t = (self.t & 0x00FF) | ((data as u16 & 0x3F) << 8);
        }else{
            self.t = (self.t & 0xFF00) | (data as u16);
            self.v = self.t;
        }
        
        self.w = !self.w;
    }

    pub fn write_data(&mut self, data: u8){
        self.write(self.v, data);
        self.increment_vram_addr();
    }

    //Might be wrong
    fn increment_vram_addr(&mut self){
        let scanline = Scanline::from(self.scanline);
        if !self.is_rendering_enabled(){
            let increment = if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
            self.v = (self.v + increment) & 0x7FFF;
        }else {
            self.increment_h();
            self.increment_v();
        }
    }

    fn is_rendering_enabled(&self) -> bool{
        self.mask & 0x10 != 0
    }

    fn is_bg_rendering_enabled(&self) -> bool{
        self.mask & 0x08 != 0
    }

    fn is_leftmost_bg_rendering_enabled(&self) -> bool{
        self.mask & 0x02 != 0
    }

    fn is_leftmost_sprite_rendering_enabled(&self) -> bool{
        self.mask & 0x04 != 0
    }

    fn sprite_pattern_table_address(&self) -> u16 {
        if self.ctrl & 0x8 != 0 { 0x1000 } else { 0 }
    }

    fn bg_pattern_table_address(&self) -> u16 {
        if self.ctrl & 0x10 != 0 { 0x1000 } else { 0 }
    }

    fn fine_y(&self) -> u16 {
        (self.v >> 12) & 7
    }

    //CORRECT
    fn increment_h(&mut self) {
        if (self.v & 0x001F) == 31 {
            self.v = (self.v & !0x001F) ^ 0x0400;
        } else {
            self.v += 1;        // Increment coarse X
        }
    }

    //CORRECT
    fn increment_v(&mut self){

        if (self.v & 0x7000) != 0x7000{
            self.v += 0x1000;
        }else{
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            }else if y == 31 {
                y = 0;
            }else{
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }

    }

    //CORRECT
    fn copy_h(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    //CoRRECT
    fn copy_v(&mut self) {
        let y_mask = 0x7000 | 0x0800 | 0x03E0;
        self.v = (self.v & !y_mask) | (self.t & y_mask);
    }
}