use core::panic;
use std::{fs::OpenOptions, io::{self, Write}};

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

#[derive(Copy, Clone)]
pub struct Sprite {
    y: u8,
    tile: u8,
    attr: u8,
    x: u8,
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            y: 0,
            tile: 0,
            attr: 0,
            x: 0,
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
    pub secondary_oam: [u8; 8], 
    pub sprite_cache: [u16; 256],
    pub sprites: Vec<Sprite>,
    pub trigger_nmi: bool,

    pub cycle: usize,
    pub scanline: usize,

    pub frame_ready: bool,
    pub frame_buffer: [u8; 256 * 240 * 3],

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

    current_sprite: usize,
    sprites_found: usize,
    sprite_attr_index: usize,
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

            oam: [Sprite { y:0, tile:0, attr:0, x:0 }; 64],
            secondary_oam: [0xFF; 8],
            sprite_cache: [0xFFFF; 256],
            sprites,
            trigger_nmi: false,

            cycle: 0,
            scanline: 0,

            frame_buffer: [0; 256 * 240 * 3],
            frame_ready: false,

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

            current_sprite: 0,
            sprites_found: 0,
            sprite_attr_index: 0,
        }
    }

    pub fn step(&mut self){
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
                    self.evaluate_sprites();
                    self.load_pixel();
                    self.load_shift_registers();
                }
                
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

    fn evaluate_sprites(&mut self) {
        let cycle = self.cycle;
        
        
        if cycle == 1 {
            self.secondary_oam = [0xFF; 8];
            return;
        }
        
        
        if cycle == 65 {

            let scanline = self.scanline as u16;
            let mut n_idx = 0;
            let mut n  = 0;
            let sp_h = self.sprite_height() as u16;
            for (i, sprite) in  self.oam.iter().enumerate() {
                let y = sprite.y as u16;
                if y <= scanline && scanline < y + sp_h {
                    self.secondary_oam[n_idx] = i as u8;
                    n_idx += 1;
                    if n_idx == 8 {
                        n = i + 1;
                        break;
                    }
                }

            }

            if n == 8 {
                let mut m = 0;
                for i in 0..64 {
                    for j in 0..4 {
                        let y = match j {
                            0 => self.oam[i].y,
                            1 => self.oam[i].tile,
                            2 => self.oam[i].attr,
                            3 => self.oam[i].x,
                            _ => unreachable!()
                        };
                        if y  as u16 <= scanline && scanline < y as u16 + sp_h {
                            self.status |= 0x20;
                        }else{
                            m = (m + 1) & 3;
                        }
                        n += 1;
                    }
                }
            }
            return;
        }
        
        
        if cycle == 257{
            self.fetch_sprite();
            return;
        }
    }

    fn fetch_sprite(&mut self) {

        self.sprite_cache = [0xFFFF; 256];

        for i in 0..8 {
            let j = self.secondary_oam[i];
            if j == 0xFF {
                break;
            }
    
            let y = self.oam[j as usize].y;
            let tile = self.oam[j as usize].tile;
            let attr = self.oam[j as usize].attr;
            let x = self.oam[j as usize].x;
    
            
            let v_flip = attr & 0x80 != 0;
            let h_flip = attr & 0x40 != 0;
            
            let y0 = self.scanline as u16 - y as u16;
            let (pt, tile_num, y) = match self.sprite_height() {
                8 => {
                    let y = if v_flip { 7 - y0 as u8 } else { y0 as u8 };
                    ((self.ctrl as u16 & 0x08) << 9, tile, y)
                }
                16 => {
                    let y = if v_flip { 15 - y0 as u8 } else { y0 as u8 };
                    (
                        (tile as u16 & 1) << 12,
                        (tile & !1u8) | (y >> 3),
                        y & 0x7
                    )
                }
                _ => unreachable!()
            };
    
            let addr = pt | ((tile_num as u16) << 4) | y as u16;
            let mut pt_lo = self.read(addr);
            let mut pt_hi = self.read(addr + 8);
    
            if h_flip {
                pt_lo = Ppu::reverse_byte(pt_lo);
                pt_hi = Ppu::reverse_byte(pt_hi);
            }
    
            let palette = attr & 3;
            let x_max = if x as usize + 8 < 256 { x as usize + 8 } else { 256 };
            
            for p in (&mut self.sprite_cache[x as usize..x_max]).iter_mut().rev() {
                if *p == 0xFFFF {
                    let sp = ((palette << 2) | ((pt_hi & 1) << 1) | (pt_lo & 1)) as u16;
                    if sp & 3 != 0 {
                        *p = ((if j == 0 {1} else {0}) << 15) | (((attr >> 5) as u16 & 1) << 8) | sp;
                    }
                }
                pt_hi >>= 1;
                pt_lo >>= 1;
            }
        }
    }

    #[inline(always)]
    fn reverse_byte(mut x: u8) -> u8 {
        x = ((x & 0xaa) >> 1) | ((x & 0x55) << 1);
        x = ((x & 0xcc) >> 2) | ((x & 0x33) << 2);
        x = ((x & 0xf0) >> 4) | ((x & 0x0f) << 4);
        x
    }

    fn get_pattern_address(&self, sprite_index: usize, row: u8) -> u16 {
        let sprite = &self.sprites[sprite_index];
        let mut pattern_addr = self.sprite_pattern_table_address();
        
        if self.sprite_height() == 16 {
            pattern_addr = ((sprite.tile as u16 & 0x01) << 12) |
                          ((sprite.tile as u16 & 0xFE) << 4);
        } else {
            pattern_addr += (sprite.tile as u16) << 4;
        }

        let final_row = if sprite.is_v_flipped() {
            (self.sprite_height() - 1).wrapping_sub(row)
        } else {
            row
        };

        pattern_addr + final_row as u16
    }

    fn update_scroll(&mut self) {
        if !self.is_rendering_enabled() {
            return;
        }
    
        
        if self.scanline < 240 || self.scanline == 261 {
            
            if self.cycle > 0 && self.cycle <= 256 {
                if self.cycle % 8 == 0 {
                    self.increment_h();
                }
            } else if self.cycle >= 321 && self.cycle <= 336 {
                if self.cycle % 8 == 0 {
                    self.increment_h();
                }
            }
    
            
            if self.cycle == 256 {
                self.increment_v();
            }
    
            
            if self.cycle == 257 {
                self.copy_h();
            }
    
            
            if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
                self.copy_v();
            }
        }
    }

    fn reload_shifters(&mut self) {
        self.pt_shifter_lo = (self.pt_shifter_lo & 0xFF00) | self.pt_latch_lo as u16;
        self.pt_shifter_hi = (self.pt_shifter_hi & 0xFF00) | self.pt_latch_hi as u16;

        self.at_latch_lo = self.at_byte & 1;
        self.at_latch_hi = self.at_byte & 2;
    }

    fn load_shift_registers(&mut self) {
        match self.cycle {
            1..=256 | 321..=336 => {

                match (self.cycle - 1) % 8 {
                    0 => { 
                        self.nt_byte = self.get_nt();
                    }
                    2 => { 
              
                        self.at_byte = self.get_attribute();
                        
                        if (self.coarse_y() & 2) != 0 {
                            self.at_byte >>= 4;
                        }

                        if (self.coarse_x() & 2) != 0 {
                            self.at_byte >>= 2;
                        }
                        
                        
                    }
                    4 => { 
                        let addr = self.bg_pattern_table_address() + (self.nt_byte as u16 * 16) + self.fine_y();
                        self.pt_latch_lo = self.read(addr);
                    }
                    6 => { 
                        let addr = self.bg_pattern_table_address() + (self.nt_byte as u16 * 16) + self.fine_y();
                        self.pt_latch_hi = self.read(addr + 8);
                    }
                    7 => {
                        self.reload_shifters();
                    }
                    _ => {}
                }
            }
            257..=320 => {
                if self.cycle == 257 {
                    self.reload_shifters();
                }
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
        
        if self.scanline >= 240 || self.cycle > 256 || self.cycle == 0 {
            return;
        }

        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;
    
        if self.is_bg_rendering_enabled() {
            
            if self.cycle >= 8 || self.is_leftmost_bg_rendering_enabled() {
                
                let fine_x = self.x & 0x7;
                let shift = (15 - fine_x) as u16;
                bg_pixel = ((((self.pt_shifter_hi >> shift) & 1) << 1) | ((self.pt_shifter_lo >> shift) & 1)) as u8;

                bg_palette = ((((self.at_shifter_hi >> (7 - fine_x)) & 1) << 1) | ((self.at_shifter_lo >> (7 - fine_x)) & 1)) << 2;
            }
        }
    
        let mut sprite_pixel = 0u8;
        let mut sprite_palette = 0u8;
        let mut sprite_priority = false;
        
        if self.is_sprite_rendering_enabled() {
            if self.cycle >= 8 || self.is_leftmost_sprite_rendering_enabled() {
                let x = self.cycle - 1;
                let p = self.sprite_cache[x];
                if p != 0xFFFF {
                    if (p >> 15 == 1) && bg_pixel != 0 && x != 0xFF {
                        self.status |= 0x40;
                    }
                    sprite_priority = (p >> 8) & 1 != 0;
                    let palette_and_pixel = p & 0xFF;
                    sprite_pixel = (palette_and_pixel & 0x03) as u8;
                    if sprite_pixel != 0 {
                        sprite_palette = ((palette_and_pixel & 0x0C) | 0x10) as u8;
                    }
                }
            }
        }

        let (final_pixel, final_palette) = match (bg_pixel, sprite_pixel) {
            (0, 0) => (0, 0), 
            (0, _) => (sprite_pixel, sprite_palette), 
            (_, 0) => (bg_pixel, bg_palette), 
            (_, _) => {
                
                if sprite_priority {
                    (bg_pixel, bg_palette) 
                } else {
                    (sprite_pixel, sprite_palette) 
                }
            }
        };

        let palette_idx = if final_pixel == 0 { 0 } else { final_palette | final_pixel };
        
        
        let color = (self.palette[palette_idx as usize] & 0x3F) as usize;
        let x = self.cycle - 1;
        let y = self.scanline;
        let idx = (y * 256 + x) * 3;

        self.frame_buffer[idx] = PALETTE[color * 3];
        self.frame_buffer[idx + 1] = PALETTE[(color * 3) + 1];
        self.frame_buffer[idx + 2] = PALETTE[(color * 3) + 2];

        self.pt_shifter_lo <<= 1;
        self.pt_shifter_hi <<= 1;
        self.at_shifter_lo = (self.at_shifter_lo << 1) | self.at_latch_lo;
        self.at_shifter_hi = (self.at_shifter_hi << 1) | self.at_latch_hi;

    }

    fn is_sprite_in_range(&self, y_coordinate: u8) -> bool {
        let diff = self.scanline as i16 - y_coordinate as i16;
        diff >= 0 && diff < self.sprite_height() as i16
    }

    fn get_nt(&mut self) -> u8{
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.read(addr)
    }

    fn get_attribute(&mut self) -> u8{
        let addr = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        self.read(addr)
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
        if (self.v & 0x001F) == 31 {
            self.v = (self.v & !0x001F) ^ 0x0400;
        } else {
            self.v += 1;        
        }
    }

    
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

    
    fn copy_h(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    
    fn copy_v(&mut self) {
        let y_mask = 0x7000 | 0x0800 | 0x03E0;
        self.v = (self.v & !y_mask) | (self.t & y_mask);
    }
}