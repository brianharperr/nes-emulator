pub static HEADER_SIZE: usize = 16;

#[derive(Debug, PartialEq)]
pub enum INesVersion{
    Unknown,
    One,
    Two
}

#[derive(Debug)]
pub enum Console{
    NES,
    VsSystem,
    Playchoice10,
    Extended
}

#[derive(Debug, Clone)]
pub enum Mirroring{
    Vertical,
    Horizontal,
    SingleScreen,
    FourScreen
}

#[derive(Debug)]
pub enum TvSystem {
    NTSC,
    PAL,
    DualCompatible,
    Dendy
}

pub struct RomHeader {
    pub nes_version : INesVersion,
    pub prg_rom_banks: u8,
    pub prg_rom_size: u32,
    pub prg_ram_size: u32,
    pub prg_nvram_size: u32,
    pub chr_rom_size: u32,
    pub chr_ram_size: u32,
    pub chr_nvram_size: u32,
    pub mapper_number: u16,
    pub submapper: u8,
    pub battery: bool,
    pub trainer: bool,
    pub mirroring: Mirroring,
    pub console: Console,
    pub tv: TvSystem
    //TODO: Add remaining iNES2.0 fields
}

impl RomHeader {
    pub fn new(data: Vec<u8>) -> Self{
        let mut nes_version = INesVersion::Unknown;

        if data[0] == 0x4E && data[1] == 0x45 && data[2] == 0x53 && data[3] == 0x1A {
            nes_version = INesVersion::One;
        }

        let flag_6 = data[6];
        let flag_7 = data[7];

        if nes_version == INesVersion::One && (flag_7 & 0x0C == 0x08) {
            nes_version = INesVersion::Two;
        }

        let battery = flag_6 & 0x02 != 0;
        let trainer = (flag_6 & 0x04) != 0;


        let (mapper_number, submapper) = match nes_version {
            INesVersion::One => {
                let lower_nibble = (flag_6 >> 4) & 0x0F;
                let upper_nibble = flag_7 & 0xF0;
                let mapper = ((upper_nibble << 4) | lower_nibble) as u16;
                (mapper, 0)
            }
            INesVersion::Two => {
                let lower_bits = ((flag_6 >> 4) | (flag_7 & 0xF0)) as u16;
                let upper_bits = (data[8] & 0x0F) as u16;
                let mapper = (upper_bits << 8) | lower_bits;

                let submapper = (data[8] >> 4) & 0x0F;
                (mapper, submapper)
            }
            _ => (0, 0)
        };
        
        let console_number = flag_7 & 0x3;
        let console = match console_number {
            0 => Console::NES,
            1 => Console::VsSystem,
            2 => Console::Playchoice10,
            3 => Console::Extended,
            _ => Console::NES
        };

        //TODO: Mirroring is determined by mapper for a few mappers.
        let mirroring = if flag_6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if flag_6 & 0x01 == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let prg_rom_banks = data[4];
        let (prg_rom_size, prg_ram_size, prg_nvram_size, chr_rom_size, chr_ram_size, chr_nvram_size) = match nes_version {
            INesVersion::One => {
                // PRG-ROM size in 16KB units
                let prg_rom = (prg_rom_banks as u32) * 16 * 1024;
                // CHR-ROM size in 8KB units
                let chr_rom = (data[5] as u32) * 8 * 1024;
                // PRG-RAM size (if present, assume 8KB)
                let prg_ram = if (flag_6 & 0x02) != 0 { 8 * 1024 } else { 0 };
                
                (
                    prg_rom,
                    prg_ram,
                    0,          // No NVRAM size in iNES 1.0
                    chr_rom,
                    0,          // No CHR-RAM size in iNES 1.0
                    0,          // No CHR-NVRAM size in iNES 1.0
                )
            }
            INesVersion::Two => {
                // Calculate sizes using NES 2.0 format
                let prg_rom = if data[4] != 0 {
                    let shift = (data[9] >> 4) & 0xF;
                    (data[4] as u32) * (1 << (shift as u32))
                } else { 0 };
    
                let chr_rom = if data[5] != 0 {
                    let shift = data[9] & 0xF;
                    (data[5] as u32) * (1 << (shift as u32))
                } else { 0 };
    
                let prg_ram = if (data[10] & 0xF) != 0 {
                    1 << ((data[10] & 0xF) as u32 + 6)
                } else { 0 };
    
                let prg_nvram = if (data[10] >> 4) != 0 {
                    1 << (((data[10] >> 4) & 0xF) as u32 + 6)
                } else { 0 };
    
                let chr_ram = if (data[11] & 0xF) != 0 {
                    1 << ((data[11] & 0xF) as u32 + 6)
                } else { 0 };
    
                let chr_nvram = if (data[11] >> 4) != 0 {
                    1 << (((data[11] >> 4) & 0xF) as u32 + 6)
                } else { 0 };
    
                (
                    prg_rom,
                    prg_ram,
                    prg_nvram,
                    chr_rom,
                    chr_ram,
                    chr_nvram,
                )
            }
            INesVersion::Unknown => {
                (0, 0, 0, 0, 0, 0)
            }
        };

        let tv = match nes_version {
            INesVersion::One => {
                if data[9] & 0x01 != 0 {
                    TvSystem::PAL
                }else{
                    TvSystem::NTSC
                }
            }
            INesVersion::Two => {
                match data[12] & 0x03 {
                    0 => TvSystem::NTSC,
                    1 => TvSystem::PAL,
                    2 => TvSystem::DualCompatible,
                    3 => TvSystem::Dendy,
                    _ => panic!("Unknown TV System")
                }
            }
            INesVersion::Unknown => TvSystem::DualCompatible
        };

        RomHeader{
            nes_version,
            prg_rom_banks,
            mapper_number,
            submapper,
            battery,
            trainer,
            console,
            mirroring,
            prg_rom_size, 
            prg_ram_size, 
            prg_nvram_size, 
            chr_rom_size, 
            chr_ram_size, 
            chr_nvram_size,
            tv
        }
    }
}