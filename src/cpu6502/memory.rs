use crate::cpu6502::MEM_SIZE;

pub struct Mem {
    memory: Vec<u8>,
}
impl Mem {
    pub fn init_mem() -> Mem {
        Mem {
            memory: vec![0; MEM_SIZE],
        }
    }
    pub fn set_byte(mut self, index: u16, val: u8) {
        self.memory[index as usize] = val;
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn dump_memory(self, print_all: bool) {
        for i in (0..MEM_SIZE).step_by(0x10) {
            let slice = &self.memory[i..i + 0x10];

            if slice.iter().any(|&x| x > 0) || print_all {
                print!("0x{i:#>04x}: ");
                for byte in slice.iter().cloned() {
                    print!("{byte:02x} ");
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
            }
        }
    }
    pub fn set_all(mut self, new_mem: Vec<u8>) {
        self.memory = new_mem;
    }
}
