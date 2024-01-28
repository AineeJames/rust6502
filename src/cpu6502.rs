use crate::utils::pause::pause_for_input;
use clap::Parser;
use log::{debug, info};
use std::{fs, usize};

const MEM_SIZE: usize = 65536;

#[derive(Debug, Clone, Copy)]
enum AddressingMode {
    Accumulator,
    Implied,
    Immediate,
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    ZeroPage,
    Relative,
    AbsoluteIndirect,
    AbsoluteIndirectX,
    AbsoluteIndirectY,
    ZeroPageX,
    ZeroPageY,
    ZeroPageIndirectIndexedX,
    ZeroPageIndirectIndexedY,
}

fn get_addressing_mode_operand_length(mode: AddressingMode) -> u8 {
    match mode {
        AddressingMode::Accumulator => 0,
        AddressingMode::Implied => 0,
        AddressingMode::Immediate => 1,
        AddressingMode::Absolute => 2,
        AddressingMode::AbsoluteXIndexed => 2,
        AddressingMode::AbsoluteYIndexed => 2,
        AddressingMode::ZeroPage => 1,
        AddressingMode::Relative => 1,
        AddressingMode::AbsoluteIndirect => 2,
        AddressingMode::AbsoluteIndirectX => 2,
        AddressingMode::AbsoluteIndirectY => 2,
        AddressingMode::ZeroPageX => 1,
        AddressingMode::ZeroPageY => 1,
        AddressingMode::ZeroPageIndirectIndexedX => 1,
        AddressingMode::ZeroPageIndirectIndexedY => 1,
    }
}

fn get_instruction_length(mode: AddressingMode) -> u8 {
    1 + get_addressing_mode_operand_length(mode)
}

struct InstructionMetadata {
    mode: AddressingMode,
    instruction_name: String,
    instruction_byte_length: u8,
}

impl InstructionMetadata {
    fn new(mode: AddressingMode, instruction_name: String) -> InstructionMetadata {
        InstructionMetadata {
            mode,
            instruction_name,
            instruction_byte_length: get_instruction_length(mode),
        }
    }
}

// TODO: at compile time lookup for instruction b
fn get_opcode_metadata(opcode: u8) -> InstructionMetadata {
    match opcode {
        // ADC
        0x69 => InstructionMetadata::new(AddressingMode::Immediate, String::from("ADC")),
        0x6d => InstructionMetadata::new(AddressingMode::Absolute, String::from("ADC")),
        0x7d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("ADC")),
        0x79 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("ADC")),
        0x65 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("ADC")),
        0x75 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("ADC")),
        0x61 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedX,
            String::from("ADC"),
        ),
        0x71 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedY,
            String::from("ADC"),
        ),

        // LDX
        0xa2 => InstructionMetadata::new(AddressingMode::Immediate, String::from("LDX")),
        0xae => InstructionMetadata::new(AddressingMode::Absolute, String::from("LDX")),
        0xbe => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("LDX")),
        0xa6 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("LDX")),
        0xb6 => InstructionMetadata::new(AddressingMode::ZeroPageY, String::from("LDX")),

        // LDY
        0xa0 => InstructionMetadata::new(AddressingMode::Immediate, String::from("LDY")),
        0xac => InstructionMetadata::new(AddressingMode::Absolute, String::from("LDY")),
        0xbc => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("LDY")),
        0xa4 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("LDY")),
        0xb4 => InstructionMetadata::new(AddressingMode::ZeroPageY, String::from("LDY")),

        // STX
        0x8e => InstructionMetadata::new(AddressingMode::Absolute, String::from("STX")),
        0x86 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("STX")),
        0x96 => InstructionMetadata::new(AddressingMode::ZeroPageY, String::from("STX")),

        // STY
        0x8c => InstructionMetadata::new(AddressingMode::Absolute, String::from("STY")),
        0x84 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("STY")),
        0x94 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("STY")),

        // CPX
        0xe0 => InstructionMetadata::new(AddressingMode::Immediate, String::from("CPX")),
        0xec => InstructionMetadata::new(AddressingMode::Absolute, String::from("CPX")),
        0xe4 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("CPX")),

        // CPY
        0xc0 => InstructionMetadata::new(AddressingMode::Immediate, String::from("CPY")),
        0xcc => InstructionMetadata::new(AddressingMode::Absolute, String::from("CPY")),
        0xc4 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("CPY")),

        // DEC
        0xce => InstructionMetadata::new(AddressingMode::Absolute, String::from("DEC")),
        0xde => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("DEC")),
        0xc6 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("DEC")),
        0xd6 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("DEC")),

        // DEX
        0xca => InstructionMetadata::new(AddressingMode::Implied, String::from("DEX")),

        // DEY
        0x88 => InstructionMetadata::new(AddressingMode::Implied, String::from("DEY")),

        // JMP
        0x4c => InstructionMetadata::new(AddressingMode::Absolute, String::from("JMP")),
        0x6c => InstructionMetadata::new(AddressingMode::AbsoluteIndirect, String::from("JMP")),

        // NOP
        0xea => InstructionMetadata::new(AddressingMode::Implied, String::from("NOP")),

        // LDA
        0xa9 => InstructionMetadata::new(AddressingMode::Immediate, String::from("LDA")),
        0xad => InstructionMetadata::new(AddressingMode::Absolute, String::from("LDA")),
        0xbd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("LDA")),
        0xb9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("LDA")),
        0xa5 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("LDA")),
        0xb5 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("LDA")),
        0xa1 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedX,
            String::from("LDA"),
        ),
        0xb1 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedY,
            String::from("LDA"),
        ),

        // STA
        0x8d => InstructionMetadata::new(AddressingMode::Absolute, String::from("STA")),
        0x9d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("STA")),
        0x99 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("STA")),
        0x85 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("STA")),
        0x95 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("STA")),
        0x81 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedX,
            String::from("STA"),
        ),
        0x91 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedY,
            String::from("STA"),
        ),

        // JSR
        0x20 => InstructionMetadata::new(AddressingMode::Absolute, String::from("JSR")),

        // RTS
        0x60 => InstructionMetadata::new(AddressingMode::Implied, String::from("RTS")),

        // CMP
        0xc9 => InstructionMetadata::new(AddressingMode::Immediate, String::from("CMP")),
        0xcd => InstructionMetadata::new(AddressingMode::Absolute, String::from("CMP")),
        0xdd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("CMP")),
        0xd9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("CMP")),
        0xc5 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("CMP")),
        0xd5 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("CMP")),
        0xc1 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedX,
            String::from("CMP"),
        ),
        0xd1 => InstructionMetadata::new(
            AddressingMode::ZeroPageIndirectIndexedY,
            String::from("CMP"),
        ),

        // BEQ
        0xf0 => InstructionMetadata::new(AddressingMode::Relative, String::from("BEQ")),

        // INC
        0xee => InstructionMetadata::new(AddressingMode::Absolute, String::from("INC")),
        0xfe => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, String::from("INC")),
        0xe6 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("INC")),
        0xf6 => InstructionMetadata::new(AddressingMode::ZeroPageX, String::from("INC")),

        // INX
        0xe8 => InstructionMetadata::new(AddressingMode::Implied, String::from("INX")),

        // INY
        0xc8 => InstructionMetadata::new(AddressingMode::Implied, String::from("INY")),
        _ => todo!("Missing instruction metadata for opcode 0x{:#>02x}", opcode),
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // 6502 hex file to run
    #[arg(short, long)]
    pub binary_file: String,

    // Print all mem even if zeroed
    #[arg(short, long, default_value_t = false)]
    pub print_all_mem: bool,

    // Print all mem even if zeroed
    #[arg(short, long, default_value_t = false)]
    pub step_debug: bool,
}

// 7  bit  0
// ---- ----
// NV1B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ (No CPU effect; see: the B flag)
// ||+------- (No CPU effect; always pushed as 1)
// |+-------- Overflow
// +--------- Negative
enum Flag {
    Negative,
    Overflow,
    Unused,
    Break,
    DecimalMode,
    Interrupt,
    Zero,
    Carry,
}

struct StatusFlags {
    n: bool,
    v: bool,
    u: bool,
    b: bool,
    d: bool,
    i: bool,
    z: bool,
    c: bool,
}
impl StatusFlags {
    fn print_status_flags_readable(&self) {
        println!("Negative Flag: {}", self.n);
        println!("Overflow Flag: {}", self.v);
        println!("Unused Flag: {}", self.u);
        println!("Break Flag: {}", self.b);
        println!("Decimal Mode: {}", self.d);
        println!("Interrupt Disable: {}", self.i);
        println!("Zero Flag: {}", self.z);
        println!("Carry Flag: {}\n", self.c);
    }
    fn set_flag(&mut self, to_set_flag: Flag, flag_state: bool) {
        match to_set_flag {
            Flag::Negative => self.n = flag_state,
            Flag::Overflow => self.v = flag_state,
            Flag::Unused => self.u = flag_state,
            Flag::Break => self.b = flag_state,
            Flag::DecimalMode => self.d = flag_state,
            Flag::Interrupt => self.i = flag_state,
            Flag::Zero => self.z = flag_state,
            Flag::Carry => self.c = flag_state,
        }
    }
}

pub struct Cpu6502 {
    pub memory: Vec<u8>,
    pub accumulator: u8,
    pub x_index: u8,
    pub y_index: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status_flags: StatusFlags,
    pub cmdline_args: Args,
}

pub fn init_cpu6502(args: Args) -> Cpu6502 {
    let mut cpu = Cpu6502 {
        cmdline_args: args,
        memory: vec![0; MEM_SIZE], // stack (0x0100, 0x01FF)
        accumulator: 0,
        x_index: 0,
        y_index: 0,
        program_counter: 0,
        stack_pointer: 0xFF,
        status_flags: StatusFlags {
            n: false,
            v: false,
            u: true,
            b: false,
            d: false,
            i: false,
            z: false,
            c: false,
        },
    };

    cpu
}

enum Index {
    X,
    Y,
}

impl Cpu6502 {
    pub fn dump_memory(&self) {
        for i in (0..MEM_SIZE).step_by(0x10) {
            let slice = &self.memory[i..i + 0x10];

            if slice.iter().any(|&x| x > 0) || self.cmdline_args.print_all_mem {
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

    pub fn print_state(&self) {
        self.status_flags.print_status_flags_readable();
        println!("X register = 0x{:#>02x}", self.x_index);
        println!("Y register = 0x{:#>02x}", self.y_index);
        println!("Accumulator = 0x{:#>04x}", self.accumulator);
        println!("Program Counter = 0x{:#>04x}", self.program_counter);
        println!("Stack Pointer = 0x{:#>02x}", self.stack_pointer);
        self.dump_memory();
    }

    pub fn load_file_into_memory(&mut self) {
        let code_result: Result<Vec<u8>, std::io::Error> =
            fs::read(self.cmdline_args.binary_file.clone());
        let code = match code_result {
            Ok(code) => code,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
        self.memory = code;
    }

    fn get_next_byte(&mut self) -> u8 {
        let instruction: u8 = *self.memory.get(self.program_counter as usize).unwrap();
        return instruction;
    }

    fn print_instruction(&mut self, instruction: &InstructionMetadata) {
        // STX (ZeroPageY) operand
        let operand = match instruction.mode {
            AddressingMode::AbsoluteXIndexed => format!(
                "${:#>04x},X",
                self.get_addr(AddressingMode::AbsoluteXIndexed)
            ),
            AddressingMode::Relative => {
                format!("${:#>04x}", self.get_addr(AddressingMode::Relative))
            }
            AddressingMode::Implied => format!(""),
            AddressingMode::Absolute => format!("${:#>04x}", self.get_addr(instruction.mode)),
            AddressingMode::AbsoluteIndirect => {
                format!("(${:#>04x})", self.get_addr(AddressingMode::Absolute))
            }
            AddressingMode::Immediate => {
                let addr = self.get_addr(instruction.mode);
                format!("#${:#>02x}", self.memory[addr])
            }
            AddressingMode::ZeroPage => format!("${:#>02x}", self.get_addr(instruction.mode)),
            AddressingMode::ZeroPageX => format!(
                "${:#>02x},X",
                self.get_addr(instruction.mode) - self.x_index as usize
            ),
            AddressingMode::ZeroPageY => format!(
                "${:#>02x},Y",
                self.get_addr(instruction.mode) - self.y_index as usize
            ),
            AddressingMode::ZeroPageIndirectIndexedX => format!(
                "$({:#>02x},X)",
                self.get_addr(instruction.mode) - self.x_index as usize
            ),
            AddressingMode::ZeroPageIndirectIndexedY => format!(
                "$({:#>02x},Y)",
                self.get_addr(instruction.mode) - self.y_index as usize
            ),
            _ => todo!(
                "Add format for addressing mode {:?} in print_instruction()",
                instruction.mode
            ),
        };
        println!(
            "\nNEXT INSTRUCTION: {} {}",
            instruction.instruction_name, operand
        );
    }

    fn get_abs_addr(&self) -> usize {
        let ll = self.memory[(self.program_counter + 1) as usize] as usize;
        let hh = self.memory[(self.program_counter + 2) as usize] as usize;
        let addr = (hh << 8) | ll;
        return addr;
    }

    fn get_zpg_addr(&mut self, index: Option<Index>) -> usize {
        let addr = match index {
            Some(Index::X) => {
                self.x_index as usize + self.memory[(self.program_counter + 1) as usize] as usize
            }
            Some(Index::Y) => {
                self.y_index as usize + self.memory[(self.program_counter + 1) as usize] as usize
            }
            None => self.memory[(self.program_counter + 1) as usize] as usize,
        };
        return addr;
    }

    fn get_zpg_indirect_addr(&mut self, index: Index) -> usize {
        let addr = match index {
            Index::X => self.memory[(self.program_counter + 1) as usize] + self.x_index as u8,
            Index::Y => self.memory[(self.program_counter + 1) as usize] + self.y_index as u8,
        };
        addr as usize
    }

    fn get_abs_indirect_addr(&mut self) -> usize {
        let mem_addr = self.get_abs_addr();
        let ll = self.memory[mem_addr] as usize;
        let hh = if (mem_addr & 0xFF) == 0xFF {
            self.memory[mem_addr & 0xFF00] as usize
        } else {
            self.memory[mem_addr + 1] as usize
        };
        let addr = (hh << 8) | ll;
        return addr;
    }

    fn get_addr(&mut self, mode: AddressingMode) -> usize {
        let addr: usize = match mode {
            AddressingMode::Implied => 0,
            AddressingMode::Relative => self.program_counter as usize + 1,
            AddressingMode::Immediate => self.program_counter as usize + 1,
            AddressingMode::Absolute => self.get_abs_addr(),
            AddressingMode::AbsoluteIndirect => self.get_abs_indirect_addr(),
            AddressingMode::AbsoluteXIndexed => self.get_abs_addr() + self.x_index as usize,
            AddressingMode::AbsoluteYIndexed => self.get_abs_addr() + self.y_index as usize,
            AddressingMode::ZeroPage => self.get_zpg_addr(None),
            AddressingMode::ZeroPageX => self.get_zpg_addr(Some(Index::X)),
            AddressingMode::ZeroPageY => self.get_zpg_addr(Some(Index::Y)),
            AddressingMode::ZeroPageIndirectIndexedX => self.get_zpg_indirect_addr(Index::X),
            AddressingMode::ZeroPageIndirectIndexedY => self.get_zpg_indirect_addr(Index::Y),
            _ => todo!("Mode not implemented in get_addr()"),
        };
        return addr;
    }

    fn ldx(&mut self, mode: AddressingMode) {
        // load from memory into x
        let addr = self.get_addr(mode);
        let val = self.memory[addr];
        self.x_index = val;
        if val == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
        if (val & 0b01000000) != 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn ldy(&mut self, mode: AddressingMode) {
        // load from memory into y
        let addr = self.get_addr(mode);
        let val = self.memory[addr];
        self.y_index = val;
        if val == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
        if (val & 0b10000000) != 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn stx(&mut self, mode: AddressingMode) {
        // store index x into memory
        let addr = self.get_addr(mode);
        self.memory[addr] = self.x_index;
    }

    fn sty(&mut self, mode: AddressingMode) {
        // store index y into memory
        let addr = self.get_addr(mode);
        self.memory[addr] = self.y_index;
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        if self.x_index >= self.memory[addr] {
            self.status_flags.set_flag(Flag::Carry, true);
        }
        if self.memory[addr] > self.x_index {
            self.status_flags.set_flag(Flag::Carry, false);
        }
        if (self.x_index - self.memory[addr]) & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
        if self.x_index == self.memory[addr] {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        if self.y_index >= self.memory[addr] {
            self.status_flags.set_flag(Flag::Carry, true);
        }
        if self.memory[addr] > self.y_index {
            self.status_flags.set_flag(Flag::Carry, false);
        }
        if (self.y_index - self.memory[addr]) & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
        if self.y_index == self.memory[addr] {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn dec(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory[addr] -= 1;
        if (self.memory[addr]) & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
        if self.memory[addr] == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn dex(&mut self) {
        self.x_index -= 1;
        if (self.x_index) & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
        if self.x_index == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn dey(&mut self) {
        self.y_index -= 1;
        if (self.y_index) & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
        if self.y_index == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn jmp(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.program_counter = addr as u16
    }

    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator = self.memory[addr];
        if self.accumulator == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
        if (self.accumulator & 0b10000000) != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory[addr] = self.accumulator;
    }

    fn jsr(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        // Store the return address onto the stack
        let return_addr = self.program_counter + 3; // Adjust for the JSR instruction size
        self.push_stack((return_addr >> 8) as u8); // High byte
        self.push_stack((return_addr & 0xFF) as u8); // Low byte
                                                     // Set the program counter to the target address
        self.program_counter = addr as u16;
    }

    fn rts(&mut self) {
        let low_byte = self.pop_stack();
        let high_byte = self.pop_stack();
        let return_addr = ((high_byte as u16) << 8) | low_byte as u16;
        self.program_counter = return_addr;
    }

    fn cmp(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory[addr];
        let result = self.accumulator.wrapping_sub(value);

        if result == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }

        if result & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }

        if value <= self.accumulator {
            self.status_flags.set_flag(Flag::Carry, true);
        } else {
            self.status_flags.set_flag(Flag::Carry, false);
        }
    }

    fn beq(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory[addr];
        println!("DEBUG: addr = 0x{:#>04x}", addr as u8);
        println!("DEBUG: offset = {}", offset as i8);
        if self.status_flags.z {
            let new_pc = self.program_counter + offset as u16;
            self.program_counter = new_pc;
            println!("DEBUG: new_pc = {}", new_pc as u16);
        } else {
            self.program_counter += 2;
        }
    }

    fn inc(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory[addr];
        let new_value = value.wrapping_add(1);
        self.memory[addr] = new_value;

        if new_value == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }

        if new_value & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }
    }

    fn inx(&mut self) {
        let result: u8 = if self.x_index == 0xFF {
            0
        } else {
            self.x_index + 1
        };

        self.x_index = result;

        if result & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }

        if result == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn iny(&mut self) {
        let result: u8 = if self.y_index == 0xFF {
            0
        } else {
            self.y_index + 1
        };

        self.y_index = result;

        if result & 0b10000000 != 0 {
            self.status_flags.set_flag(Flag::Negative, true);
        } else {
            self.status_flags.set_flag(Flag::Negative, false);
        }

        if result == 0 {
            self.status_flags.set_flag(Flag::Zero, true);
        } else {
            self.status_flags.set_flag(Flag::Zero, false);
        }
    }

    fn push_stack(&mut self, value: u8) {
        let stack_addr = 0x0100 | (self.stack_pointer as u16);
        self.memory[stack_addr as usize] = value;
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 | (self.stack_pointer as u16);
        return self.memory[stack_addr as usize];
    }

    pub fn run(&mut self) {
        let rvec: u16 = (self.memory[0xfffd] as u16) << 8 | self.memory[0xfffc] as u16;
        self.program_counter = rvec;
        loop {
            self.print_state();

            let cur_opcode = self.get_next_byte();
            let instruction: InstructionMetadata = get_opcode_metadata(cur_opcode);
            self.print_instruction(&instruction);

            if self.cmdline_args.step_debug {
                pause_for_input();
            }

            let instruction_name = instruction.instruction_name.as_str();
            match instruction_name {
                "ADC" => self.adc(instruction.mode),
                "STX" => self.stx(instruction.mode),
                "STY" => self.sty(instruction.mode),
                "LDX" => self.ldx(instruction.mode),
                "LDY" => self.ldy(instruction.mode),
                "CPX" => self.cpx(instruction.mode),
                "CPY" => self.cpy(instruction.mode),
                "DEC" => self.dec(instruction.mode),
                "DEX" => self.dex(),
                "DEY" => self.dey(),
                "JMP" => self.jmp(instruction.mode),
                "NOP" => {}
                "LDA" => self.lda(instruction.mode),
                "STA" => self.sta(instruction.mode),
                "JSR" => self.jsr(instruction.mode),
                "RTS" => self.rts(),
                "CMP" => self.cmp(instruction.mode),
                "BEQ" => self.beq(instruction.mode),
                "INC" => self.inc(instruction.mode),
                "INX" => self.inx(),
                "INY" => self.iny(),
                _ => todo!("Add instruction {instruction_name} to run()"),
            }
            // increment program counter by instruction length
            if !matches!(instruction_name, "JMP" | "JSR" | "RTS" | "BEQ") {
                self.program_counter += instruction.instruction_byte_length as u16;
            }
        }
    }

    fn adc(&mut self, mode: AddressingMode) {
        // TODO: Decimal mode if status register
        // has decimal mode flag set need to treat
        // hex as decimal for example 0x65 == 65
        let addr = self.get_addr(mode);
        debug!("Address being used to ADC {:#>04x}", addr);
        let carry_add = self.status_flags.c as u8;
        let mem_val: u16 = self.memory[addr as usize] as u16;
        let overflow_flag_before_add: bool = (self.accumulator & (1 << 7)) == 1;
        let sum = mem_val + self.accumulator as u16 + carry_add as u16;
        self.accumulator = sum as u8;
        if sum > 255 {
            self.status_flags.set_flag(Flag::Carry, true);
        } else {
            self.status_flags.set_flag(Flag::Carry, false);
        }
        let overflow_flag_after_add: bool = (self.accumulator & (1 << 7)) == 1;
        if overflow_flag_before_add != overflow_flag_after_add {
            self.status_flags.set_flag(Flag::Overflow, true);
        } else {
            self.status_flags.set_flag(Flag::Overflow, false);
        }
    }
}
