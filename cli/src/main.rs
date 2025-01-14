extern crate nes_cpu;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

use nes_cpu::rom::Rom;
use nes_cpu::Nes;
use sdl_wrapper::SDLWrapper;

mod sdl_wrapper;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Missing ROM file path.");
    }

    let filepath = &args[1];
    let file = File::open(filepath).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data).expect("Failed to read file");

    let rom = Rom::new(data);
    debug_rom(&rom);
    
    let mut nes = Nes::new(nes_cpu::SystemVersion::NTSC);
    nes.set_rom(rom);
    // nes.set_debug_mode();
    nes.on();
    // nes.set_start(0xC000);
    // nes.run();
    let mut wrapper = SDLWrapper::new(nes);
    wrapper.run();
}

fn debug_rom(rom: &Rom){
    println!("iNES Version: {:?}", rom.header.nes_version);
    println!("PRG ROM SIZE: {}", rom.header.prg_rom_size);
    println!("PRG RAM SIZE: {}", rom.header.prg_ram_size);
    println!("PRG NRAM SIZE: {}", rom.header.prg_nvram_size);
    println!("CHR ROM SIZE: {}", rom.header.chr_rom_size);
    println!("CHR RAM SIZE: {}", rom.header.chr_ram_size);
    println!("CHR NRAM SIZE: {}", rom.header.chr_nvram_size);
    println!("Mapper: {}", rom.header.mapper_number);
    println!("Uses battery: {}", rom.header.battery);
    println!("Trainer present: {}", rom.header.trainer);
    println!("Console: {:?}", rom.header.console);
    println!("Mirroring: {:?}", rom.header.mirroring);
    println!("TV System: {:?}", rom.header.tv);
}