
pub mod header;

use header::{RomHeader, HEADER_SIZE};

use crate::{mapper::{Mapper, MapperFactory}, memory::Memory};

pub struct Rom {
    pub header: RomHeader,
    pub mapper: Box<dyn Mapper>
}

impl Rom {

    pub fn new(data: Vec<u8>) -> Self {

        let header = RomHeader::new(data[0..HEADER_SIZE].to_vec());

        let mapper = MapperFactory::select(&header, data);

        Rom {
            header,
            mapper,
        }
    }
}