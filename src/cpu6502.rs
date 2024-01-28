use crate::utils::pause::pause_for_input;

use clap::Parser;

use std::fs;
use std::io;
use std::io::prelude::*;

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
    ZeroPageIndexedIndirect,
    ZeroPageIndirectIndexedY,
}

fn get_addressing_mode_operand_length(mode: AddressingMode) -> u8 {
    match mode {
        AddressingMode::Accumulator => 0,
        AddressingMode::Implied => 1,
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
        AddressingMode::ZeroPageIndexedIndirect => 1,
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
            mode: mode,
            instruction_name: instruction_name,
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

        // LDX
        0xa2 => InstructionMetadata::new(AddressingMode::Immediate, String::from("LDX")),
        0xae => InstructionMetadata::new(AddressingMode::Absolute, String::from("LDX")),
        0xbe => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, String::from("LDX")),
        0xa6 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("LDX")),
        0xb6 => InstructionMetadata::new(AddressingMode::ZeroPageY, String::from("LDX")),

        // STX
        0x8e => InstructionMetadata::new(AddressingMode::Absolute, String::from("STX")),
        0x86 => InstructionMetadata::new(AddressingMode::ZeroPage, String::from("STX")),
        0x96 => InstructionMetadata::new(AddressingMode::ZeroPageY, String::from("STX")),
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
        stack_pointer: 0,
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
    // reset vec
    cpu.memory[0xfffc] = 0x06;
    cpu.memory[0xfffd] = 0x00;

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
        println!("Stack Pointer = {}", self.stack_pointer);
        self.dump_memory();
    }

    pub fn load_file_into_memory(&mut self, org: usize) {
        let code_result: Result<Vec<u8>, std::io::Error> =
            fs::read(self.cmdline_args.binary_file.clone());
        let code = match code_result {
            Ok(code) => code,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
        self.memory[org..org + code.len()].copy_from_slice(&code);
    }

    fn get_next_byte(&mut self) -> u8 {
        let instruction: u8 = *self.memory.get(self.program_counter as usize).unwrap();
        return instruction;
    }

    fn print_instruction(&mut self, instruction: &InstructionMetadata) {
        // STX (ZeroPageY) operand
        let operand = match instruction.mode {
            AddressingMode::Absolute => format!("${:#>04x}", self.get_addr(instruction.mode)),
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

    fn get_addr(&mut self, mode: AddressingMode) -> usize {
        let addr: usize = match mode {
            AddressingMode::Immediate => self.program_counter as usize + 1,
            AddressingMode::Absolute => self.get_abs_addr(),
            AddressingMode::AbsoluteXIndexed => self.get_abs_addr() + self.x_index as usize,
            AddressingMode::AbsoluteYIndexed => self.get_abs_addr() + self.y_index as usize,
            AddressingMode::ZeroPage => self.get_zpg_addr(None),
            AddressingMode::ZeroPageX => self.get_zpg_addr(Some(Index::X)),
            AddressingMode::ZeroPageY => self.get_zpg_addr(Some(Index::Y)),
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

    fn stx(&mut self, mode: AddressingMode) {
        // store index x into memory
        let addr = self.get_addr(mode);
        self.memory[addr] = self.x_index;
    }

    pub fn run(&mut self) {
        let rvec: u16 = (self.memory[0xfffc] as u16) << 8 | self.memory[0xfffd] as u16;
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
                "LDX" => self.ldx(instruction.mode),
                _ => todo!("Add instruction {instruction_name} to run()"),
            }
            // increment program counter by instruction length
            self.program_counter += instruction.instruction_byte_length as u16;
        }
    }

    fn adc(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let carry_add = self.status_flags.c as u8;
        let mem_before_add: u16 = self.memory[addr as usize] as u16;
        let sum = mem_before_add + self.accumulator as u16 + carry_add as u16;
        if sum > 255 {
            self.accumulator = sum as u8;
            self.status_flags.c = true;
        } else {
            self.accumulator = sum as u8;
        }
        self.status_flags.c = false;
    }
}
