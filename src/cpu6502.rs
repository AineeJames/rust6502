use crate::utils::pause::pause_for_input;
use clap::Parser;
use log::debug;
use std::time::{Duration, Instant};
use std::{fs, usize};
pub mod memory;

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

#[derive(Debug)]
enum Instruction {
    ADC,
    LDX,
    LDY,
    STX,
    STY,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    JMP,
    NOP,
    LDA,
    STA,
    JSR,
    RTS,
    CMP,
    BEQ,
    INC,
    INX,
    INY,
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
    instruction_type: Instruction,
    instruction_byte_length: u8,
}

impl InstructionMetadata {
    fn new(mode: AddressingMode, instruction: Instruction) -> InstructionMetadata {
        InstructionMetadata {
            mode,
            instruction_type: instruction,
            instruction_byte_length: get_instruction_length(mode),
        }
    }
}

// TODO: at compile time lookup for instruction b
fn get_opcode_metadata(opcode: u8) -> InstructionMetadata {
    match opcode {
        // ADC
        0x69 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::ADC),
        0x6d => InstructionMetadata::new(AddressingMode::Absolute, Instruction::ADC),
        0x7d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::ADC),
        0x79 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::ADC),
        0x65 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::ADC),
        0x75 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::ADC),
        0x61 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::ADC)
        }
        0x71 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::ADC)
        }

        // LDX
        0xa2 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDX),
        0xae => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDX),
        0xbe => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDX),
        0xa6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDX),
        0xb6 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::LDX),

        // LDY
        0xa0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDY),
        0xac => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDY),
        0xbc => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDY),
        0xa4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDY),
        0xb4 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::LDY),

        // STX
        0x8e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STX),
        0x86 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STX),
        0x96 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::STX),

        // STY
        0x8c => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STY),
        0x84 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STY),
        0x94 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::STY),

        // CPX
        0xe0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CPX),
        0xec => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CPX),
        0xe4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CPX),

        // CPY
        0xc0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CPY),
        0xcc => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CPY),
        0xc4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CPY),

        // DEC
        0xce => InstructionMetadata::new(AddressingMode::Absolute, Instruction::DEC),
        0xde => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::DEC),
        0xc6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::DEC),
        0xd6 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::DEC),

        // DEX
        0xca => InstructionMetadata::new(AddressingMode::Implied, Instruction::DEX),

        // DEY
        0x88 => InstructionMetadata::new(AddressingMode::Implied, Instruction::DEY),

        // JMP
        0x4c => InstructionMetadata::new(AddressingMode::Absolute, Instruction::JMP),
        0x6c => InstructionMetadata::new(AddressingMode::AbsoluteIndirect, Instruction::JMP),

        // NOP
        0xea => InstructionMetadata::new(AddressingMode::Implied, Instruction::NOP),

        // LDA
        0xa9 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDA),
        0xad => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDA),
        0xbd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::LDA),
        0xb9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDA),
        0xa5 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDA),
        0xb5 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::LDA),
        0xa1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::LDA)
        }
        0xb1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::LDA)
        }

        // STA
        0x8d => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STA),
        0x9d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::STA),
        0x99 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::STA),
        0x85 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STA),
        0x95 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::STA),
        0x81 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::STA)
        }
        0x91 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::STA)
        }

        // JSR
        0x20 => InstructionMetadata::new(AddressingMode::Absolute, Instruction::JSR),

        // RTS
        0x60 => InstructionMetadata::new(AddressingMode::Implied, Instruction::RTS),

        // CMP
        0xc9 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CMP),
        0xcd => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CMP),
        0xdd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::CMP),
        0xd9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::CMP),
        0xc5 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CMP),
        0xd5 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::CMP),
        0xc1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::CMP)
        }
        0xd1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::CMP)
        }

        // BEQ
        0xf0 => InstructionMetadata::new(AddressingMode::Relative, Instruction::BEQ),

        // INC
        0xee => InstructionMetadata::new(AddressingMode::Absolute, Instruction::INC),
        0xfe => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::INC),
        0xe6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::INC),
        0xf6 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::INC),

        // INX
        0xe8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::INX),

        // INY
        0xc8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::INY),
        _ => todo!("Missing instruction metadata for opcode 0x{:#>02x}", opcode),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // 6502 hex file to run
    #[arg(help = "Input file", required = true)]
    pub binary_file: String,

    // Print all mem even if zeroed
    #[arg(
        help = "Print all memory each iteration",
        short,
        long,
        default_value_t = false
    )]
    pub print_all_mem: bool,

    // Step through program
    #[arg(
        help = "Step through instruction by instruction",
        short,
        long,
        default_value_t = false
    )]
    pub step_debug: bool,

    // No printing of cpu regs, mem, etc...
    #[arg(
        help = "Dont print any debug info",
        short,
        long,
        default_value_t = false
    )]
    pub no_print: bool,

    // Print instructions per second
    #[arg(
        help = "Print out instructions per second messages",
        short,
        long,
        default_value_t = false
    )]
    pub instrumentation: bool,
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

pub struct StatusFlags {
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
    pub memory: memory::Mem,
    pub accumulator: u8,
    pub x_index: u8,
    pub y_index: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status_flags: StatusFlags,
    pub cmdline_args: Args,
    pub start_time: Instant,
    pub instructions_executed: usize,
}

pub fn init_cpu6502(args: Args) -> Cpu6502 {
    let cpu = Cpu6502 {
        instructions_executed: 0,
        start_time: Instant::now(),
        cmdline_args: args,
        memory: memory::Mem::init_mem(),
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
    pub fn print_state(&self) {
        if self.cmdline_args.no_print {
            return;
        }
        self.status_flags.print_status_flags_readable();
        println!("X register = 0x{:#>02x}", self.x_index);
        println!("Y register = 0x{:#>02x}", self.y_index);
        println!("Accumulator = 0x{:#>04x}", self.accumulator);
        println!("Program Counter = 0x{:#>04x}", self.program_counter);
        println!("Stack Pointer = 0x{:#>02x}", self.stack_pointer);
        self.memory.dump_memory(self.cmdline_args.print_all_mem);
    }

    pub fn load_file_into_memory(&mut self) {
        let code_result: Result<Vec<u8>, std::io::Error> =
            fs::read(self.cmdline_args.binary_file.clone());
        let code = match code_result {
            Ok(code) => code,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
        self.memory.set_all(code);
    }

    fn get_next_byte(&mut self) -> u8 {
        let instruction: u8 = *self.memory.get(self.program_counter as usize).unwrap();
        return instruction;
    }

    fn print_instruction(&mut self, instruction: &InstructionMetadata) {
        // STX (ZeroPageY) operand
        if self.cmdline_args.no_print {
            return;
        }

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
                format!("#${:#>02x}", self.memory.get_byte(addr))
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
            "\nNEXT INSTRUCTION: {:?} {}",
            instruction.instruction_type, operand
        );
    }

    fn get_abs_addr(&self) -> usize {
        let ll = self.memory.get_byte((self.program_counter + 1) as usize) as usize;
        let hh = self.memory.get_byte((self.program_counter + 2) as usize) as usize;
        let addr = (hh << 8) | ll;
        return addr;
    }

    fn get_zpg_addr(&mut self, index: Option<Index>) -> usize {
        let addr = match index {
            Some(Index::X) => {
                self.x_index as usize
                    + self.memory.get_byte((self.program_counter + 1) as usize) as usize
            }
            Some(Index::Y) => {
                self.y_index as usize
                    + self.memory.get_byte((self.program_counter + 1) as usize) as usize
            }
            None => self.memory.get_byte((self.program_counter + 1) as usize) as usize,
        };
        return addr;
    }

    fn get_zpg_indirect_addr(&mut self, index: Index) -> usize {
        let addr = match index {
            Index::X => {
                self.memory.get_byte((self.program_counter + 1) as usize) + self.x_index as u8
            }
            Index::Y => {
                self.memory.get_byte((self.program_counter + 1) as usize) + self.y_index as u8
            }
        };
        addr as usize
    }

    fn get_abs_indirect_addr(&mut self) -> usize {
        let mem_addr = self.get_abs_addr();
        let ll = self.memory.get_byte(mem_addr) as usize;
        let hh = if (mem_addr & 0xFF) == 0xFF {
            self.memory.get_byte(mem_addr & 0xFF00) as usize
        } else {
            self.memory.get_byte(mem_addr + 1) as usize
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
        let val = self.memory.get_byte(addr);
        self.x_index = val;
        self.status_flags.set_flag(Flag::Zero, val == 0);
        self.status_flags
            .set_flag(Flag::Negative, val & 0b1000000 != 0);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        // load from memory into y
        let addr = self.get_addr(mode);
        let val = self.memory.get_byte(addr);
        self.y_index = val;
        self.status_flags.set_flag(Flag::Zero, val == 0);
        self.status_flags
            .set_flag(Flag::Negative, val & 0b1000000 != 0);
    }

    fn stx(&mut self, mode: AddressingMode) {
        // store index x into memory
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.x_index);
    }

    fn sty(&mut self, mode: AddressingMode) {
        // store index y into memory
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.y_index);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        if self.x_index >= self.memory.get_byte(addr) {
            self.status_flags.set_flag(Flag::Carry, true);
        }
        if self.memory.get_byte(addr) > self.x_index {
            self.status_flags.set_flag(Flag::Carry, false);
        }
        self.status_flags.set_flag(
            Flag::Negative,
            (self.x_index - self.memory.get_byte(addr)) & 0b1000000 != 0,
        );
        self.status_flags
            .set_flag(Flag::Zero, self.x_index == self.memory.get_byte(addr));
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        if self.y_index >= self.memory.get_byte(addr) {
            self.status_flags.set_flag(Flag::Carry, true);
        }
        if self.memory.get_byte(addr) > self.y_index {
            self.status_flags.set_flag(Flag::Carry, false);
        }
        self.status_flags.set_flag(
            Flag::Negative,
            (self.y_index - self.memory.get_byte(addr)) & 0b1000000 != 0,
        );
        self.status_flags
            .set_flag(Flag::Zero, self.y_index == self.memory.get_byte(addr));
    }

    fn dec(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory.decrement_mem(addr);
        self.status_flags
            .set_flag(Flag::Negative, self.memory.get_byte(addr) & 0b10000000 != 0);
        self.status_flags
            .set_flag(Flag::Zero, self.memory.get_byte(addr) == 0);
    }

    fn dex(&mut self) {
        self.x_index -= 1;
        self.status_flags
            .set_flag(Flag::Negative, self.x_index & 0b10000000 != 0);
        self.status_flags.set_flag(Flag::Zero, self.x_index == 0);
    }

    fn dey(&mut self) {
        self.y_index -= 1;
        self.status_flags
            .set_flag(Flag::Negative, self.y_index & 0b10000000 != 0);
        self.status_flags.set_flag(Flag::Zero, self.y_index == 0);
    }

    fn jmp(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.program_counter = addr as u16
    }

    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator = self.memory.get_byte(addr);
        self.status_flags
            .set_flag(Flag::Zero, self.accumulator == 0);
        self.status_flags
            .set_flag(Flag::Negative, (self.accumulator & 0b10000000) != 0);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.accumulator);
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
        let value = self.memory.get_byte(addr);
        let result = self.accumulator.wrapping_sub(value);

        self.status_flags.set_flag(Flag::Zero, result == 0);
        self.status_flags
            .set_flag(Flag::Negative, result & 0b10000000 != 0);
        self.status_flags
            .set_flag(Flag::Carry, value <= self.accumulator);
    }

    fn beq(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if self.status_flags.z {
            let new_pc = self.program_counter + 2 + offset as u16;
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn inc(&mut self, mode: AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory.get_byte(addr);
        let new_value = value.wrapping_add(1);
        self.memory.set_byte(addr, new_value);

        self.status_flags.set_flag(Flag::Zero, new_value == 0);
        self.status_flags
            .set_flag(Flag::Negative, new_value & 0b10000000 != 0);
    }

    fn inx(&mut self) {
        let result: u8 = if self.x_index == 0xFF {
            0
        } else {
            self.x_index + 1
        };

        self.x_index = result;

        self.status_flags
            .set_flag(Flag::Negative, result & 0b10000000 != 0);
        self.status_flags.set_flag(Flag::Zero, result == 0);
    }

    fn iny(&mut self) {
        let result: u8 = if self.y_index == 0xFF {
            0
        } else {
            self.y_index + 1
        };

        self.y_index = result;

        self.status_flags
            .set_flag(Flag::Negative, result & 0b10000000 != 0);
        self.status_flags.set_flag(Flag::Zero, result == 0);
    }

    fn push_stack(&mut self, value: u8) {
        let stack_addr = 0x0100 | (self.stack_pointer as u16);
        self.memory.set_byte(stack_addr as usize, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 | (self.stack_pointer as u16);
        return self.memory.get_byte(stack_addr as usize);
    }

    pub fn run(&mut self) {
        let rvec: u16 =
            (self.memory.get_byte(0xfffd) as u16) << 8 | self.memory.get_byte(0xfffc) as u16;
        self.program_counter = rvec;
        loop {
            self.print_state();
            let cur_opcode = self.get_next_byte();
            let instruction: InstructionMetadata = get_opcode_metadata(cur_opcode);
            self.print_instruction(&instruction);

            if self.cmdline_args.step_debug {
                pause_for_input();
            }

            match instruction.instruction_type {
                Instruction::ADC => self.adc(instruction.mode),
                Instruction::STX => self.stx(instruction.mode),
                Instruction::STY => self.sty(instruction.mode),
                Instruction::LDX => self.ldx(instruction.mode),
                Instruction::LDY => self.ldy(instruction.mode),
                Instruction::CPX => self.cpx(instruction.mode),
                Instruction::CPY => self.cpy(instruction.mode),
                Instruction::DEC => self.dec(instruction.mode),
                Instruction::DEX => self.dex(),
                Instruction::DEY => self.dey(),
                Instruction::JMP => self.jmp(instruction.mode),
                Instruction::NOP => {}
                Instruction::LDA => self.lda(instruction.mode),
                Instruction::STA => self.sta(instruction.mode),
                Instruction::JSR => self.jsr(instruction.mode),
                Instruction::RTS => self.rts(),
                Instruction::CMP => self.cmp(instruction.mode),
                Instruction::BEQ => self.beq(instruction.mode),
                Instruction::INC => self.inc(instruction.mode),
                Instruction::INX => self.inx(),
                Instruction::INY => self.iny(),
                _ => todo!(
                    "Add instruction {:?} to run()",
                    instruction.instruction_type
                ),
            }
            // increment program counter by instruction length
            if !matches!(
                instruction.instruction_type,
                Instruction::JMP | Instruction::JSR | Instruction::RTS | Instruction::BEQ
            ) {
                self.program_counter += instruction.instruction_byte_length as u16;
            }

            self.instructions_executed += 1;
            if self.cmdline_args.instrumentation && (self.instructions_executed % 10000 == 0) {
                let duration = self.start_time.elapsed().as_nanos();
                // we have been executing for this long
                let instructions_per_second =
                    (self.instructions_executed * 1_000_000_000) as u128 / (duration as u128);
                println!(
                    "Currently executing at {:?} instructions per second",
                    instructions_per_second
                );
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
        let mem_val: u16 = self.memory.get_byte(addr as usize) as u16;
        let overflow_flag_before_add: bool = (self.accumulator & (1 << 7)) == 1;
        let sum = mem_val + self.accumulator as u16 + carry_add as u16;
        self.accumulator = sum as u8;
        self.status_flags.set_flag(Flag::Carry, sum > 255);
        let overflow_flag_after_add: bool = (self.accumulator & (1 << 7)) == 1;
        self.status_flags.set_flag(
            Flag::Overflow,
            overflow_flag_after_add != overflow_flag_before_add,
        );
    }
}
