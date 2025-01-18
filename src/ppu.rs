use core::panic;
use std::{fs::OpenOptions, io::{self, Write}, iter::Scan};

use crate::{memory::Memory, rom::{header::{Mirroring, HEADER_SIZE}, Rom}};

const fn nth_bit(x: u16, n: u8) -> u16 {
    (x >> n) & 1
}

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

#[derive(Copy, Clone)]
pub struct Sprite {
    id: u8,
    y: u8,
    tile: u8,
    attr: u8,
    x: u8,
    pt_lo: u8,
    pt_hi: u8
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            id: 64,
            y: 0xFF,
            tile: 0xFF,
            attr: 0xFF,
            x: 0xFF,
            pt_lo: 0xFF,
            pt_hi: 0xFF
        }
    }

    pub fn is_v_flipped(&self) -> bool {
        self.attr & 0x80 != 0
    }

    pub fn is_h_flipped(&self) -> bool {
        self.attr & 0x40 != 0
    }

    pub fn priority(&self) -> bool {
        self.attr & 0x20 != 0
    }

    pub fn palette(&self) -> u8 {
        self.attr & 0x03
    }
}

const PPU_VRAM_SIZE: usize = 0x800; 
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

    pub oam: [Sprite; 64],
    pub secondary_oam: [Sprite; 8], 
    pub sprite_cache: [Sprite; 8],
    pub sprites: Vec<Sprite>,
    pub trigger_nmi: bool,

    pub cycle: usize,
    pub scanline: usize,

    pub frame_ready: bool,
    pub frame_buffer: [u8; 256 * 240 * 3],

    addr_latch: u16,

    nt_byte: u8,
    at_byte: u8,
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

        let mut sprites = Vec::with_capacity(0);
        for _ in 0..8 {
            sprites.push(Sprite::new());
        }

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

            oam: [Sprite::new(); 64],
            secondary_oam: [Sprite::new(); 8],
            sprite_cache: [Sprite::new(); 8],
            sprites,
            trigger_nmi: false,

            cycle: 0,
            scanline: 0,

            frame_buffer: [0; 256 * 240 * 3],
            frame_ready: false,

            addr_latch: 0,

            nt_byte: 0,
            at_byte: 0,
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

    fn cycle(&mut self, s: Scanline) {
        let cycle = self.cycle;
        if s == Scanline::VBlank && cycle == 1 {
            self.status |= 0x80;
            if self.ctrl & 0x80 != 0 {
                self.trigger_nmi = true;
            }
        }else if s == Scanline::PostRender && cycle == 0 {
            self.frame_ready = true;
        }else if s == Scanline::PreRender || s == Scanline::Visible {
            
            match cycle {
                1 => {
                    self.clear_oam();
                    if s == Scanline::PreRender {
                        self.status &= 0x1F; 
                    }
                },
                257 => self.eval_sprites(),
                321 => self.load_sprites(),
                _ => {}
            }

            
            match cycle {
                2..=255 | 322..=337 => {
                    self.load_pixel();
                    match cycle % 8 {
                        1 => {
                            self.addr_latch = self.nt_addr();
                            self.reload_shifters();
                        },
                        2 => self.nt_byte = self.read(self.addr_latch),
                        3 => self.addr_latch = self.at_addr(),
                        4 => {
                            self.at_byte = self.read(self.addr_latch);
                            if self.coarse_y() & 2 != 0 { self.at_byte >>= 4 }
                            if self.coarse_x() & 2 != 0 { self.at_byte >>= 2 }
                        }
                        5 => self.addr_latch = self.pt_addr(),
                        6 => self.pt_latch_lo = self.read(self.addr_latch),
                        7 => self.addr_latch += 8,
                        0 => {
                            self.pt_latch_hi = self.read(self.addr_latch);
                            self.increment_h();
                        },
                        _ => unreachable!()
                    }
                },
                256 => {
                    self.load_pixel();
                    self.pt_latch_hi = self.read(self.addr_latch);
                    self.increment_v();
                },
                257 => {
                    self.load_pixel();
                    self.reload_shifters();
                    self.copy_h();
                },
                280..=304 => if s == Scanline::PreRender { self.copy_v() },

                1 => {
                    self.addr_latch = self.nt_addr();
                    if s == Scanline::PreRender {
                        self.status &= 0x7F;
                    }
                },
                321 | 339 => self.addr_latch = self.nt_addr(),
                338 => self.nt_byte = self.read(self.addr_latch),
                340 => {
                    self.nt_byte = self.read(self.addr_latch);
                    if s == Scanline::PreRender && self.odd_frame {
                        self.cycle += 1;
                    }
                },
                _ => {}
            }
        }
    }

    pub fn step(&mut self){
        match self.scanline {
            0..=239 => self.cycle(Scanline::Visible),
            240 => self.cycle(Scanline::PostRender),
            241 => self.cycle(Scanline::VBlank),
            261 => self.cycle(Scanline::PreRender),
            _ => {}
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle %= CYCLERS_PER_SCANLINE;
            self.scanline += 1;
            if self.scanline >= NUM_SCANLINES {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame
            }
        }
    }

    fn shift(&mut self){
        self.at_shifter_lo = (self.at_shifter_lo << 1) | self.at_latch_lo;
        self.at_shifter_hi = (self.at_shifter_hi << 1) | self.at_latch_hi;
        self.pt_shifter_lo <<= 1;
        self.pt_shifter_hi <<= 1;
    }

    fn clear_oam(&mut self) {
        for i in 0..8 {
            self.secondary_oam[i].y = 0xFF;
            self.secondary_oam[i].tile = 0xFF;
            self.secondary_oam[i].attr = 0xFF;
            self.secondary_oam[i].x = 0xFF;
        }
    }

    fn eval_sprites(&mut self) {

        let mut n = 0;
        for i in 0..64 {
            let line: i16 = if self.scanline == 261 { -1 } else { self.scanline as i16 } - self.oam[i].y as i16;
            let spr_height = self.sprite_height() as i16;
            if line >= 0 && line < spr_height {
                self.secondary_oam[n].id = i as u8;
                self.secondary_oam[n].y = self.oam[i].y;
                self.secondary_oam[n].tile = self.oam[i].tile;
                self.secondary_oam[n].attr = self.oam[i].attr;
                self.secondary_oam[n].x = self.oam[i].x;

                n += 1;
                if n >= 8 {
                    self.status |= 0x20;
                    return;
                }
            }
        }
    }

    fn load_sprites(&mut self) {
        for i in 0..8 {

            self.sprite_cache[i] = self.secondary_oam[i];

            let mut addr: u16;
            let sprite_height = self.sprite_height();

            if sprite_height == 16 {
                addr = ((self.sprite_cache[i].tile as u16 & 1) * 0x1000) + ((self.sprite_cache[i].tile as u16 & !1) * 16);
            }else{
                addr = self.sprite_pattern_table_address() + (self.sprite_cache[i].tile as u16 * 16);
            }

            let mut sprite_y = self.scanline.wrapping_sub(self.sprite_cache[i].y as usize) % sprite_height as usize;
            if self.sprite_cache[i].attr & 0x80 != 0 {
                sprite_y ^= sprite_height as usize - 1;
            }
            addr += sprite_y as u16 + (sprite_y as u16 & 8);

            self.sprite_cache[i].pt_lo = self.read(addr);
            self.sprite_cache[i].pt_hi = self.read(addr + 8);
        }
    }

    #[inline]
    fn reload_shifters(&mut self) {
        self.pt_shifter_lo = (self.pt_shifter_lo & 0xFF00) | self.pt_latch_lo as u16;
        self.pt_shifter_hi = (self.pt_shifter_hi & 0xFF00) | self.pt_latch_hi as u16;

        self.at_latch_lo = self.at_byte & 1;
        self.at_latch_hi = self.at_byte & 2;
    }

    fn load_pixel(&mut self) {
        
        if self.cycle < 2 {
            return; 
        }
        let x = self.cycle - 2;
    
        let mut palette = 0u8;
        let mut obj_palette = 0u8;
        let mut obj_priority = false;
    
        if self.scanline < 240 && x < 256 {
            
            if self.is_bg_rendering_enabled() && (x >= 8 || self.is_leftmost_bg_rendering_enabled()) {
                let fine_x = self.x & 0x7;
                palette = (nth_bit(self.pt_shifter_hi, 15 - fine_x) << 1) as u8
                    | nth_bit(self.pt_shifter_lo, 15 - fine_x) as u8;
                if palette != 0 {
                    palette |= ((nth_bit(self.at_shifter_hi as u16, 7 - fine_x) << 1) as u8
                        | nth_bit(self.at_shifter_lo as u16, 7 - fine_x) as u8)
                        << 2;
                }
            }
    
            if self.is_sprite_rendering_enabled() && (x >= 8 || self.is_leftmost_sprite_rendering_enabled()) {
                for i in (0..8).rev() {
                    if self.sprite_cache[i].id == 64 {
                        continue; 
                    }
    
                    let sprite_x = x.wrapping_sub(self.sprite_cache[i].x as usize);
                    if sprite_x >= 8 {
                        continue; 
                    }
    
                    let mut sprite_x_adjusted = sprite_x;
                    if self.sprite_cache[i].attr & 0x40 != 0 {
                        sprite_x_adjusted ^= 7; 
                    }
    
                    let sprite_palette = ((nth_bit(self.sprite_cache[i].pt_hi as u16, 7 - sprite_x_adjusted as u8) << 1)
                        | nth_bit(self.sprite_cache[i].pt_lo as u16, 7 - sprite_x_adjusted as u8)) as u8;
                    if sprite_palette == 0 {
                        continue; 
                    }
    
                    if self.sprite_cache[i].id == 0 && palette != 0 && x != 255 {
                        self.status |= 0x40; 
                    }
    
                    let final_sprite_palette =
                        sprite_palette | ((self.sprite_cache[i].attr & 0x03) << 2);
                    obj_palette = final_sprite_palette + 16;
                    obj_priority = self.sprite_cache[i].attr & 0x20 != 0;
                }
            }
    
            
            if obj_palette != 0 && (palette == 0 || !obj_priority) {
                palette = obj_palette;
            }
    
            
            if !self.is_rendering_enabled() {
                palette = 0;
            }
    
            
            let color = (self.palette[palette as usize] & 0x3F) as usize;
            let idx = (self.scanline * 256 + x) * 3;
    
            self.frame_buffer[idx] = PALETTE[color * 3];
            self.frame_buffer[idx + 1] = PALETTE[color * 3 + 1];
            self.frame_buffer[idx + 2] = PALETTE[color * 3 + 2];
        }
    
        self.shift();
    }

    #[inline]
    fn nt_addr(&self) -> u16{
        0x2000 | (self.v & 0xFFF)
    }

    #[inline]
    fn at_addr(&self) -> u16{
        0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)
    }

    #[inline]
    fn pt_addr(&self) -> u16{
        self.bg_pattern_table_address() + (self.nt_byte as u16 * 16) + self.fine_y()
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let mut m_addr = addr & 0x3FFF; 

        match m_addr {
            0x0000..0x2000 => {
                self.rom.mapper.read(m_addr)
            }
            0x2000..0x3F00 => {
                let v_addr = self.map_vram_addr(m_addr);
                self.vram.read(v_addr)
            }
            0x3F00..0x4000 => {
                
                m_addr = (m_addr - 0x3F00) % 0x20;
                self.palette[m_addr as usize]
            }
            _ => self.open_bus
        }
    }

    fn append_to_file(&self, filename: &str, content: &str) -> io::Result<()> {
        
        let mut file = OpenOptions::new()
        .create(true)  
        .append(true)  
        .open(filename)?;
    
        
        file.write_all(content.as_bytes())?;
        Ok(())
    }
    
    fn write(&mut self, addr: u16, data: u8){
        let mut m_addr = addr & 0x3FFF; 
        self.open_bus = data; 

        match m_addr {
            0x0000..0x2000 => {
                
            }
            0x2000..0x3000 => {
                
                let mirr_addr = self.map_vram_addr(m_addr);
                self.vram.write(mirr_addr, data);
            }
            0x3000..0x3F00 => {
                self.write(addr - 0x1000, data);
            }
            0x3F00..0x4000 => {
                
                m_addr = (m_addr - 0x3F00) % 0x20;
                
                
                self.palette[m_addr as usize] = data;
                
                if m_addr % 4 == 0 {
                    self.palette[m_addr as usize ^ 0x10] = data;
                }
            }
            _ => panic!("PPU write outside of addressable memory: {:X} => 0x{:X}", data, addr)
        }
    }

    fn map_vram_addr(&mut self, addr: u16) -> u16 {
        
        let addr = addr & 0x3FFF;
    
        
        if addr < 0x2000 {
            panic!("map_vram_addr called with non-nametable address");
        }
        
        
        let addr = if addr >= 0x3000 { addr - 0x1000 } else { addr };
        
        
        let nametable = ((addr - 0x2000) >> 10) & 0x3;
        let offset = (addr - 0x2000) & 0x3FF;  
        
        
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
    
        
        (mapped_table * 0x400) + offset
    }
    
    
    pub fn read_status(&mut self) -> u8{
        let data = self.status;
        self.status &= !0x80; 
        self.w = false;
        self.open_bus = (data & 0xE0) | (self.open_bus & 0x1F);
        data
    }

    
    pub fn read_oam(&self) -> u8{
        0xFF
    }

    pub fn read_data(&mut self) -> u8{
        let data = if (self.v & 0x3FFF) >= 0x3F00 {
            
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
        
        let old_ctrl = self.ctrl;
        self.ctrl = data;
        if old_ctrl & 0x80 == 0 && self.ctrl & 0x80 == 1 && self.status & 0x80 == 1 {
            self.trigger_nmi = true;
        }

        self.t &= 0xF3FF;
        self.t |= (data as u16 & 0x3) << 10;
    }

    pub fn write_mask(&mut self, data: u8){
        self.mask = data;
    }

    pub fn write_oamaddr(&mut self, data: u8){
        self.oamaddr = data;
    }

    
    pub fn write_oamdata(&mut self, data: u8) {
        let sprite_index = (self.oamaddr as usize) / 4;  
        let byte_offset = (self.oamaddr as usize) % 4;   
        
        if sprite_index < self.oam.len() {
            match byte_offset {
                0 => self.oam[sprite_index].y = data,
                1 => self.oam[sprite_index].tile = data,
                2 => self.oam[sprite_index].attr = data,
                3 => self.oam[sprite_index].x = data,
                _ => unreachable!()
            }
        }
        
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

    pub fn write_addr(&mut self, data: u8) {
        if !self.w {
            self.t = (self.t & 0x00FF) | ((data as u16 & 0x3F) << 8);
        } else {
            self.t = (self.t & 0xFF00) | (data as u16);
            self.v = self.t;
        }
        self.w = !self.w;
    }

    pub fn write_data(&mut self, data: u8){
        self.write(self.v, data);
        self.increment_vram_addr();
    }

    
    fn increment_vram_addr(&mut self){
        let increment = if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
        self.v = (self.v + increment) & 0x7FFF;
    }

    fn is_rendering_enabled(&self) -> bool{
        self.mask & 0x18 != 0
    }

    fn is_sprite_rendering_enabled(&self) -> bool{
        self.mask & 0x10 != 0
    }

    fn sprite_height(&self) -> u8 {
        if self.ctrl & 0x20 != 0 { 16 } else { 8 }
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

    fn coarse_y(&self) -> u16 {
        (self.v & 0x3E0) >> 5
    }

    fn coarse_x(&self) -> u16 {
        self.v & 0x1F
    }
    
    fn increment_h(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }

        if (self.v & 0x001F) == 31 {
            self.v = (self.v & !0x001F) ^ 0x0400;
        } else {
            self.v += 1;        
        }
    }

    
    fn increment_v(&mut self){

        if !self.is_rendering_enabled() {
            return;
        }

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

    
    fn copy_h(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    
    fn copy_v(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
        let y_mask = 0x7000 | 0x0800 | 0x03E0;
        self.v = (self.v & !y_mask) | (self.t & y_mask);
    }
}