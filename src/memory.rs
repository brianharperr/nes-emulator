pub struct Memory {
	pub data: Vec<u8>
}

impl Memory {
	pub fn new(vec: Vec<u8>) -> Self {
		Memory{ data: vec }
	}

	pub fn clear(&mut self) {
		for i in 0..self.capacity() {
			self.data[i as usize] = 0;
		}
	}

	pub fn capacity(&self) -> u32 {
		self.data.len() as u32
	}

	pub fn read(&self, address: u16) -> u8 {
		self.data[address as usize]
	}

	pub fn write(&mut self, address: u16, value: u8) {
		self.data[address as usize] = value;
	}
	
}