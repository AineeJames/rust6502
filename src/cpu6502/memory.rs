use crate::cpu6502::MEM_SIZE;
use colored::Colorize;
use log::debug;

pub enum MemMap {
    CHROUT = 0xFE00,
    CHRIN = 0xFE01,
    NOMAP,
}

impl MemMap {
    pub fn from_index(index: usize) -> MemMap {
        match index {
            0xFE00 => MemMap::CHROUT,
            0xFE01 => MemMap::CHRIN,
            _ => MemMap::NOMAP,
        }
    }
}

pub struct Mem {
    memory: Vec<u8>,
}
impl Mem {
    pub fn init_mem() -> Mem {
        Mem {
            memory: vec![0; MEM_SIZE],
        }
    }
    pub fn set_byte(&mut self, index: usize, val: u8) {
        debug!("set addr 0x{:#>04x} to 0x{:#>02x}", index, val);
        self.memory[index as usize] = val;
        match MemMap::from_index(index) {
            MemMap::CHROUT => {
                if let Some(char) = char::from_u32(val as u32) {
                    print!("{}", char);
                }
            }
            _ => {}
        }
    }

    pub fn get_byte(&self, index: usize) -> u8 {
        self.memory[index as usize]
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn decrement_mem(&mut self, index: usize) {
        self.set_byte(index, self.memory[index].wrapping_sub(1));
    }

    pub fn dump_memory(&self, print_all: bool, pc: u16, sp: u8) {
        let mut new_zero_line: bool = true;
        println!("Memory:");
        for i in (0..MEM_SIZE).step_by(0x10) {
            let slice = &self.memory[i..i + 0x10];
            if slice.iter().any(|&x| x > 0) || print_all {
                print!("0x{i:#>04x}: ");
                for (offset, byte) in slice.iter().enumerate() {
                    if i + offset == pc as usize {
                        print!("{}", format!("{:02x}", byte).blue().underline());
                    } else if i + offset == sp as usize | 0x0100 {
                        print!("{}", format!("{:02x}", byte).yellow().underline());
                    } else {
                        print!("{byte:02x}");
                    }
                    print!(" ");
                }

                for &byte in slice {
                    if byte.is_ascii() && byte.is_ascii_graphic() {
                        let c: char = byte as char;
                        print!("{c}")
                    } else {
                        print!(".")
                    }
                }
                print!("\n");
                new_zero_line = false;
            } else {
                if new_zero_line == false {
                    println!("*");
                    new_zero_line = true;
                }
            }
        }
    }

    pub fn set_all(&mut self, new_mem: Vec<u8>) {
        self.memory = new_mem;
    }
}
