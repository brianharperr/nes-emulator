use rom::header::RomHeader;

use crate::{mappers::{m0::Mapper0, m1::Mapper1}, rom};

#[derive(Clone)]
pub struct MapperFactory;

impl MapperFactory {
    pub fn select(header: &RomHeader, data: Vec<u8>) -> Box<dyn Mapper> {
        match header.mapper_number {
            0 => Box::new(Mapper0::new(&header, data)),
            1 => Box::new(Mapper1::new(&header, data)),
            _ => panic!("Mapper not supported {}", header.mapper_number)
        }
    }
}
pub trait Mapper {
    fn map(&self, addr: u16) -> u16;
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}
