
use std::{fs::OpenOptions, io::{self, Write}};

use crate::SystemVersion;
use super::{bus::Bus, instructions::{AddressingMode, Instruction, OPCODE_TABLE}};

const NTSC_CLOCK_FREQ: f32 = 1.789773;
const PAL_CLOCK_FREQ: f32 = 1.662607;
const DENDY_CLOCK_FREQ: f32 = 1.773448;
const ARGENTINA_FAMICLONE_CLOCK_FREQ: f32 = 1.787806;
const BRAZIL_FAMICLONE_CLOCK_FREQ: f32 = 1.791028;

const NMI_ADDR: u16 = 0xFFFA;
const RESET_ADDR: u16 = 0xFFFC;
const IRQ_ADDR: u16 = 0xFFFE;

#[derive(Debug, Copy, Clone)]
pub enum StatusFlag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    InterruptDisable = 0b0000_0100,
    Decimal = 0b0000_1000,
    Break = 0b0001_0000,
    BreakIrq = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

pub enum Interrupt {
    NMI,
    RESET,
    IRQ,
    BRK,
}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub p: u8,

    clock_period: f32,

    pub update_interrupt_disable: (bool, u8),
    pub bus: Bus,

    //Debugging
    pub debug_mode: bool,
    opcode: u8,
    operand: Vec<u8>,
    db_a: u8,
    db_x: u8,
    db_y: u8,
    db_pc: u16,
    db_sp: u8,
    db_p: u8
}

impl Cpu {
    pub fn new(version: SystemVersion) -> Self{

        let clock_speed = match version {
            SystemVersion::NTSC | SystemVersion::RGB => {
                NTSC_CLOCK_FREQ
            }
            SystemVersion::PAL => PAL_CLOCK_FREQ,
            SystemVersion::Dendy => DENDY_CLOCK_FREQ,
            SystemVersion::BrazilFamiclone => BRAZIL_FAMICLONE_CLOCK_FREQ,
            SystemVersion::ArgentinaFamiclone => ARGENTINA_FAMICLONE_CLOCK_FREQ
        };
        let clock_period = 1.0 / (clock_speed * 1_000_000.0);
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xFFFC,
            sp: 0,
            p: 0x24,

            clock_period,
            update_interrupt_disable: (false, 0),
            bus: Bus::new(),

            debug_mode: false,
            opcode: 0,
            operand: vec![],
            db_a: 0,
            db_x: 0,
            db_y: 0,
            db_pc: 0,
            db_sp: 0,
            db_p: 0
        }
    }

    pub fn set_debug_mode(&mut self, value: bool){
        self.debug_mode = value;
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
    
    fn pad_to_width(&self, str: String, width: usize) -> String {
        let len = str.len();
        if len < width {
            let pad = " ".repeat(width - len);
            format!("{}{}", str, pad)
        } else {
            str.clone()
        }
    }

    pub fn step(&mut self){

        if self.bus.dma_transfer.0 {
            let bank = self.bus.dma_transfer.1;
            for i in 0..256 {
                let addr = bank as u16 * 0x100 + i;
                let data = self.read_byte(addr);
                self.bus.ppu.write_oamdata(data);
            }
            self.bus.cycles += 514;
            self.bus.dma_transfer = (false, 0);
        }

        if self.update_interrupt_disable.0 {
            self.set_flag(StatusFlag::InterruptDisable, self.update_interrupt_disable.1 != 0);
            self.update_interrupt_disable = (false, 0);
        }

        if self.debug_mode {
            self.db_a = self.a;
            self.db_x = self.x;
            self.db_y = self.y;
            self.db_pc = self.pc;
            self.db_sp = self.sp;
            self.db_p = self.p;
        }

        let instruction = self.fetch_instruction();
        //REFACTOR: FETCH OPERAND FIRST
        let page_cross_cycle = (instruction.function)(self, instruction.mode);
        let cycles = instruction.min_cycles + page_cross_cycle;


        if self.debug_mode {
            let operands_str = self.operand.iter()
                .map(|op| format!("{:02X}", op))
                .collect::<Vec<String>>()
                .join(" ");

            let output_str = format!(
                "{:04X}  {:02X} {:<42}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU: {}, {} CYC:{}\n",
                self.db_pc,
                self.opcode,
                self.pad_to_width(operands_str, 42),
                self.db_a,
                self.db_x,
                self.db_y,
                self.db_p,
                self.db_sp,
                self.bus.ppu.scanline, self.bus.ppu.cycle, self.bus.cycles
            );
            match self.append_to_file("debug.log", &output_str) {
                Ok(_) => (),
                Err(e) => eprintln!("Error writing to file: {}", e),
            }
            self.operand.clear();
        }

        for _ in 0..cycles * 3 {
            self.bus.ppu.step();
            if self.bus.ppu.trigger_nmi {
                self.bus.ppu.trigger_nmi = false;
                self.interrupt(Interrupt::NMI);
            }
        }

        self.bus.cycles += u64::from(cycles);
        //std::thread::sleep(std::time::Duration::from_secs_f32(cycles as f32 * self.clock_period));
    }


    fn get_test_result(&mut self) -> String{
        let mut idx = 0x6004;
        let mut result = Vec::new();

        loop {
            let curr = self.read_byte(idx);
            if curr == 0 {
                break;
            }
            result.push(curr);
            idx += 1;
        }

        String::from_utf8(result).expect("Invalid UTF-8 sequence")
    }

    pub fn reset(&mut self){
        self.bus.reset = true;
        self.pc = self.read_word(RESET_ADDR);
        self.sp = self.sp.wrapping_sub(3);
        self.bus.cycles = 7;
        self.set_flag(StatusFlag::InterruptDisable, true);
        for _ in 0..self.bus.cycles * 3 {
            self.bus.ppu.step();
        }
    }


    pub fn interrupt(&mut self, interrupt: Interrupt){
        match interrupt {
            Interrupt::BRK => {
                self.pc = self.pc.wrapping_add(1);
                for b in self.pc.to_be_bytes() {
                    self.stack_push(b);
                }
                self.set_flag(StatusFlag::Break, true);
                self.set_flag(StatusFlag::BreakIrq, true);
                self.stack_push(self.p);
                self.set_flag(StatusFlag::Break, false);
                self.set_flag(StatusFlag::InterruptDisable, true);
                self.pc = self.read_word(IRQ_ADDR);
            }
            Interrupt::IRQ => {

            }
            Interrupt::NMI => {
                let pc_bytes = self.pc.to_be_bytes();
                self.stack_push(pc_bytes[0]);
                self.stack_push(pc_bytes[1]);

                self.set_flag(StatusFlag::Break, false);
                self.set_flag(StatusFlag::BreakIrq, true);
                self.stack_push(self.p);
                self.set_flag(StatusFlag::InterruptDisable, true);

                self.pc = self.read_word(NMI_ADDR);
            }
            Interrupt::RESET => {
                self.reset();
            }
        }
    }


    fn fetch_instruction(&mut self) -> Instruction {
        let opcode = self.read_byte(self.pc);
        if self.debug_mode {
            self.opcode = opcode;
        }

        self.inc_pc();
        OPCODE_TABLE[opcode as usize]
    }

    pub fn fetch_operand_addr(&mut self, mode: AddressingMode) -> (u16, u8) {
        
        match mode {
            AddressingMode::Absolute => {
                let lo = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let hi = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let addr = (hi << 8) | lo;

                if self.debug_mode {
                    self.operand.push(lo as u8);
                    self.operand.push(hi as u8);
                }

                (addr, 0)
            }
            AddressingMode::AbsoluteX => {
                let lo = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let hi = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let base_addr = (hi << 8) | lo;
                let addr = base_addr.wrapping_add(self.x as u16);

                if self.debug_mode {
                    self.operand.push(lo as u8);
                    self.operand.push(hi as u8);
                }

                (addr, self.page_boundary_cycle(addr, base_addr))
            }
            AddressingMode::AbsoluteY => {
                let lo = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let hi = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let base_addr = (hi << 8) | lo;
                let addr = base_addr.wrapping_add(self.y as u16);

                if self.debug_mode {
                    self.operand.push(lo as u8);
                    self.operand.push(hi as u8);
                }

                (addr, self.page_boundary_cycle(addr, base_addr))
            }
            AddressingMode::Accumulator => (0,0),
            AddressingMode::Immediate => (0,0), //Use fetch_operand for immediate
            AddressingMode::Implied => (0,0),
            AddressingMode::Indirect => {
                // Read 16-bit address from PC
                let addr_lo = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let addr_hi = self.read_byte(self.pc) as u16;
                self.inc_pc();
                let addr = (addr_hi << 8) | addr_lo;
                
                // Handle hardware bug: if address is $xxFF, high byte is read from $xx00
                let hi_addr = if (addr_lo & 0xFF) == 0xFF {
                    addr & 0xFF00
                } else {
                    addr.wrapping_add(1)
                };
                
                // Read actual target address
                let target_lo = self.read_byte(addr) as u16;
                let target_hi = self.read_byte(hi_addr) as u16;
                
                if self.debug_mode {
                    self.operand.push(addr_lo as u8);
                    self.operand.push(addr_hi as u8);
                }
                
                ((target_hi << 8) | target_lo, 0)
            },
            AddressingMode::IndirectX => {
                // Read zero-page address
                let zp_addr = self.read_byte(self.pc);
                self.inc_pc();
                
                if self.debug_mode {
                    self.operand.push(zp_addr);
                }
                
                // Add X register with zero-page wrap
                let effective_zp = zp_addr.wrapping_add(self.x);
                
                // Read 16-bit address from zero page
                let target_lo = self.read_byte(effective_zp as u16) as u16;
                let target_hi = self.read_byte(effective_zp.wrapping_add(1) as u16) as u16;
                
                ((target_hi << 8) | target_lo, 0)
            },
            AddressingMode::IndirectY => {
                // Read zero-page address
                let zp_addr = self.read_byte(self.pc);
                self.inc_pc();
                
                if self.debug_mode {
                    self.operand.push(zp_addr);
                }
                
                // Read 16-bit address from zero page
                let base_lo = self.read_byte(zp_addr as u16) as u16;
                let base_hi = self.read_byte(zp_addr.wrapping_add(1) as u16) as u16;
                let base_addr = (base_hi << 8) | base_lo;
                
                // Add Y register to the indirect address
                let final_addr = base_addr.wrapping_add(self.y as u16);
                
                (final_addr, self.page_boundary_cycle(final_addr, base_addr))
            },
            AddressingMode::Relative => {
                let offset = self.read_byte(self.pc) as i8;  // Fetch the signed offset
                self.inc_pc();
                let addr = (self.pc as i16 + offset as i16) as u16;  // Add offset to the current PC

                if self.debug_mode {
                    self.operand.push(offset as u8);
                }

                (addr, self.page_boundary_cycle(self.pc, addr))
            },
            AddressingMode::ZeroPage => {
                let addr = self.read_byte(self.pc) as u16;  // Fetch the address (only low byte)
                self.inc_pc();

                if self.debug_mode {
                    self.operand.push(addr as u8);
                }

                (addr, 0) // Return the address as the operand
            },
            AddressingMode::ZeroPageX => {
                let addr = self.read_byte(self.pc) as u16;  // Fetch the address (only low byte)
                self.inc_pc();
                let addr_x = addr + self.x as u16;  // Add X register to the address

                if self.debug_mode {
                    self.operand.push(addr as u8);
                }

                let addr_x_wrapped = addr_x & 0xFF;
                (addr_x_wrapped, 0)
            },
            AddressingMode::ZeroPageY => {
                let addr = self.read_byte(self.pc) as u16;  // Fetch the address (only low byte)
                self.inc_pc();
                let addr_y = addr + self.y as u16;  // Add Y register to the address

                if self.debug_mode {
                    self.operand.push(addr as u8);
                }

                let addr_y_wrapped = addr_y & 0xFF;
                (addr_y_wrapped, 0)
            }
        }
    }

    pub fn fetch_operand(&mut self) -> u8 {
        let addr = self.pc;
        self.inc_pc();

        let byte = self.bus.read(addr);

        if self.debug_mode {
            self.operand.push(byte as u8);
        }

        byte
    }
    
    pub fn read_byte(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = u16::from(self.read_byte(addr));
        let hi = u16::from(self.read_byte(addr + 1));
        (hi << 8) | lo
    }

    pub fn set_zero_negative_flag(&mut self, data: u8){
        self.set_flag(StatusFlag::Zero, data == 0);
        self.set_flag(StatusFlag::Negative, (data & 0x80) != 0);      
    }

    pub fn set_flag(&mut self, flag: StatusFlag, value: bool){
        if value {
            self.p |= flag as u8;
        }else{
            self.p &= !(flag as u8);
        }
    }

    fn page_boundary_cycle(&self, addr1: u16, addr2: u16) -> u8 {
        if (addr1 & 0xFF00) != (addr2 & 0xFF00) { 1 } else { 0 }
    }

    pub fn get_carry_bit(&self) -> u8 {
        self.p & 0x1u8
    }

    fn inc_pc(&mut self){
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | self.sp as u16;
        self.bus.read(addr)
    }

    pub fn stack_push(&mut self, value: u8) {
        let address = 0x0100 | self.sp as u16;
        self.bus.write(address, value);
        self.sp = self.sp.wrapping_sub(1);
    }
}