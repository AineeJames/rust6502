use crate::utils::pause::pause_for_input;
use clap::Parser;
use log::debug;
use std::time::{Duration, Instant};
use std::{fs, usize};
pub mod memory;
pub mod operation;
pub mod status_reg;

const MEM_SIZE: usize = 65536;

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

pub struct Cpu6502 {
    pub memory: memory::Mem,
    pub accumulator: u8,
    pub x_index: u8,
    pub y_index: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status_flags: status_reg::StatusFlags,
    pub cmdline_args: Args,
    pub start_time: Instant,
    pub instructions_executed: u128,
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
        status_flags: status_reg::StatusFlags {
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

    fn print_instruction(&mut self, instruction: &operation::InstructionMetadata) {
        // STX (ZeroPageY) operand
        if self.cmdline_args.no_print {
            return;
        }

        let operand = match instruction.mode {
            operation::AddressingMode::AbsoluteXIndexed => format!(
                "${:#>04x},X",
                self.get_addr(operation::AddressingMode::AbsoluteXIndexed)
            ),
            operation::AddressingMode::Relative => {
                format!(
                    "${:#>04x}",
                    self.get_addr(operation::AddressingMode::Relative)
                )
            }
            operation::AddressingMode::Implied => format!(""),
            operation::AddressingMode::Absolute => {
                format!("${:#>04x}", self.get_addr(instruction.mode))
            }
            operation::AddressingMode::AbsoluteIndirect => {
                format!(
                    "(${:#>04x})",
                    self.get_addr(operation::AddressingMode::Absolute)
                )
            }
            operation::AddressingMode::Immediate => {
                let addr = self.get_addr(instruction.mode);
                format!("#${:#>02x}", self.memory.get_byte(addr))
            }
            operation::AddressingMode::ZeroPage => {
                format!("${:#>02x}", self.get_addr(instruction.mode))
            }
            operation::AddressingMode::ZeroPageX => format!(
                "${:#>02x},X",
                self.get_addr(instruction.mode) - self.x_index as usize
            ),
            operation::AddressingMode::ZeroPageY => format!(
                "${:#>02x},Y",
                self.get_addr(instruction.mode) - self.y_index as usize
            ),
            operation::AddressingMode::ZeroPageIndirectIndexedX => format!(
                "$({:#>02x},X)",
                self.get_addr(instruction.mode) - self.x_index as usize
            ),
            operation::AddressingMode::ZeroPageIndirectIndexedY => format!(
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

    fn get_addr(&mut self, mode: operation::AddressingMode) -> usize {
        let addr: usize = match mode {
            operation::AddressingMode::Implied => 0,
            operation::AddressingMode::Relative => self.program_counter as usize + 1,
            operation::AddressingMode::Immediate => self.program_counter as usize + 1,
            operation::AddressingMode::Absolute => self.get_abs_addr(),
            operation::AddressingMode::AbsoluteIndirect => self.get_abs_indirect_addr(),
            operation::AddressingMode::AbsoluteXIndexed => {
                self.get_abs_addr() + self.x_index as usize
            }
            operation::AddressingMode::AbsoluteYIndexed => {
                self.get_abs_addr() + self.y_index as usize
            }
            operation::AddressingMode::ZeroPage => self.get_zpg_addr(None),
            operation::AddressingMode::ZeroPageX => self.get_zpg_addr(Some(Index::X)),
            operation::AddressingMode::ZeroPageY => self.get_zpg_addr(Some(Index::Y)),
            operation::AddressingMode::ZeroPageIndirectIndexedX => {
                self.get_zpg_indirect_addr(Index::X)
            }
            operation::AddressingMode::ZeroPageIndirectIndexedY => {
                self.get_zpg_indirect_addr(Index::Y)
            }
            _ => todo!("Mode not implemented in get_addr()"),
        };
        return addr;
    }

    fn ldx(&mut self, mode: operation::AddressingMode) {
        // load from memory into x
        let addr = self.get_addr(mode);
        let val = self.memory.get_byte(addr);
        self.x_index = val;
        self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, val & 0b1000000 != 0);
    }

    fn ldy(&mut self, mode: operation::AddressingMode) {
        // load from memory into y
        let addr = self.get_addr(mode);
        let val = self.memory.get_byte(addr);
        self.y_index = val;
        self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, val & 0b1000000 != 0);
    }

    fn stx(&mut self, mode: operation::AddressingMode) {
        // store index x into memory
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.x_index);
    }

    fn sty(&mut self, mode: operation::AddressingMode) {
        // store index y into memory
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.y_index);
    }

    fn cpx(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        if self.x_index >= self.memory.get_byte(addr) {
            self.status_flags.set_flag(status_reg::Flag::Carry, true);
        }
        if self.memory.get_byte(addr) > self.x_index {
            self.status_flags.set_flag(status_reg::Flag::Carry, false);
        }
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            (self.x_index - self.memory.get_byte(addr)) & 0b1000000 != 0,
        );
        self.status_flags.set_flag(
            status_reg::Flag::Zero,
            self.x_index == self.memory.get_byte(addr),
        );
    }

    fn cpy(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        if self.y_index >= self.memory.get_byte(addr) {
            self.status_flags.set_flag(status_reg::Flag::Carry, true);
        }
        if self.memory.get_byte(addr) > self.y_index {
            self.status_flags.set_flag(status_reg::Flag::Carry, false);
        }
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            (self.y_index - self.memory.get_byte(addr)) & 0b1000000 != 0,
        );
        self.status_flags.set_flag(
            status_reg::Flag::Zero,
            self.y_index == self.memory.get_byte(addr),
        );
    }

    fn dec(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory.decrement_mem(addr);
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.memory.get_byte(addr) & 0b10000000 != 0,
        );
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.memory.get_byte(addr) == 0);
    }

    fn dex(&mut self) {
        self.x_index -= 1;
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.x_index & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.x_index == 0);
    }

    fn dey(&mut self) {
        self.y_index -= 1;
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.y_index & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.y_index == 0);
    }

    fn jmp(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.program_counter = addr as u16
    }

    fn lda(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator = self.memory.get_byte(addr);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            (self.accumulator & 0b10000000) != 0,
        );
    }

    fn sta(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.memory.set_byte(addr, self.accumulator);
    }

    fn jsr(&mut self, mode: operation::AddressingMode) {
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

    fn cmp(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory.get_byte(addr);
        let result = self.accumulator.wrapping_sub(value);

        self.status_flags
            .set_flag(status_reg::Flag::Zero, result == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, result & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Carry, value <= self.accumulator);
    }

    fn beq(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if self.status_flags.z {
            let new_pc = (self.program_counter + 2).wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn bne(&mut self, mode: operation::AddressingMode) {
        // TODO! why did i have to get rid of +2 to get correct
        // address
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if !self.status_flags.z {
            debug!("Taking bne branch");
            //let new_pc = self.program_counter + offset as u16;
            let new_pc = (self.program_counter + 2).wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            debug!("Not taking bne branch");
            self.program_counter += 2;
        }
    }

    fn bcc(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if !self.status_flags.c {
            debug!("Taking bcs branch");
            let new_pc = (self.program_counter + 2).wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2
        }
    }

    fn bcs(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if self.status_flags.c {
            debug!("Taking bcs branch");
            let new_pc = (self.program_counter + 2).wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2
        }
    }

    fn inc(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory.get_byte(addr);
        let new_value = value.wrapping_add(1);
        self.memory.set_byte(addr, new_value);

        self.status_flags
            .set_flag(status_reg::Flag::Zero, new_value == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, new_value & 0b10000000 != 0);
    }

    fn inx(&mut self) {
        let result: u8 = if self.x_index == 0xFF {
            0
        } else {
            self.x_index + 1
        };

        self.x_index = result;

        self.status_flags
            .set_flag(status_reg::Flag::Negative, result & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, result == 0);
    }

    fn iny(&mut self) {
        let result: u8 = if self.y_index == 0xFF {
            0
        } else {
            self.y_index + 1
        };

        self.y_index = result;

        self.status_flags
            .set_flag(status_reg::Flag::Negative, result & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, result == 0);
    }

    fn txa(&mut self) {
        self.accumulator = self.x_index;
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.accumulator & 0b10000000 != 0,
        );
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0)
    }

    fn tya(&mut self) {
        self.accumulator = self.y_index;
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.accumulator & 0b10000000 != 0,
        );
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0)
    }

    fn txs(&mut self) {
        self.push_stack(self.x_index);
    }

    fn tsx(&mut self) {
        let prev_x = self.x_index;
        self.x_index = self.pop_stack();

        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.x_index & 0b10000000 != 0);

        self.status_flags.set_flag(
            status_reg::Flag::Zero,
            self.x_index != prev_x && self.x_index == 0,
        )
    }

    fn sbc(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory.get_byte(addr);
        let acc_before_sub = self.accumulator;
        self.accumulator = self.accumulator.wrapping_sub(value);
        // wrapped around
        self.status_flags
            .set_flag(status_reg::Flag::Carry, acc_before_sub < self.accumulator);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.accumulator & (1 << 7) != 0);
        // TODO! dont know how to do overflow flag
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
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
            let instruction: operation::InstructionMetadata =
                operation::get_opcode_metadata(cur_opcode);
            self.print_instruction(&instruction);

            if self.cmdline_args.step_debug {
                pause_for_input();
            }

            match instruction.instruction_type {
                operation::Instruction::ADC => self.adc(instruction.mode),
                operation::Instruction::STX => self.stx(instruction.mode),
                operation::Instruction::STY => self.sty(instruction.mode),
                operation::Instruction::LDX => self.ldx(instruction.mode),
                operation::Instruction::LDY => self.ldy(instruction.mode),
                operation::Instruction::CPX => self.cpx(instruction.mode),
                operation::Instruction::CPY => self.cpy(instruction.mode),
                operation::Instruction::DEC => self.dec(instruction.mode),
                operation::Instruction::DEX => self.dex(),
                operation::Instruction::DEY => self.dey(),
                operation::Instruction::JMP => self.jmp(instruction.mode),
                operation::Instruction::NOP => {}
                operation::Instruction::LDA => self.lda(instruction.mode),
                operation::Instruction::STA => self.sta(instruction.mode),
                operation::Instruction::SBC => self.sbc(instruction.mode),
                operation::Instruction::JSR => self.jsr(instruction.mode),
                operation::Instruction::RTS => self.rts(),
                operation::Instruction::CMP => self.cmp(instruction.mode),
                operation::Instruction::BEQ => self.beq(instruction.mode),
                operation::Instruction::BNE => self.bne(instruction.mode),
                operation::Instruction::INC => self.inc(instruction.mode),
                operation::Instruction::INX => self.inx(),
                operation::Instruction::INY => self.iny(),
                operation::Instruction::TXS => self.txs(),
                operation::Instruction::TSX => self.tsx(),
                operation::Instruction::TXA => self.txa(),
                operation::Instruction::TYA => self.tya(),
                operation::Instruction::CLC => {
                    self.status_flags.set_flag(status_reg::Flag::Carry, false)
                }
                operation::Instruction::CLD => self
                    .status_flags
                    .set_flag(status_reg::Flag::DecimalMode, false),
                operation::Instruction::CLI => self
                    .status_flags
                    .set_flag(status_reg::Flag::Interrupt, false),
                operation::Instruction::CLV => self
                    .status_flags
                    .set_flag(status_reg::Flag::Overflow, false),
                operation::Instruction::SEC => {
                    self.status_flags.set_flag(status_reg::Flag::Carry, true)
                }
                operation::Instruction::SED => self
                    .status_flags
                    .set_flag(status_reg::Flag::DecimalMode, true),
                operation::Instruction::SEI => self
                    .status_flags
                    .set_flag(status_reg::Flag::Interrupt, true),
                operation::Instruction::BCS => self.bcs(instruction.mode),
                operation::Instruction::BCC => self.bcc(instruction.mode),
                _ => todo!(
                    "Add instruction {:?} to run()",
                    instruction.instruction_type
                ),
            }
            // increment program counter by instruction length
            if !matches!(
                instruction.instruction_type,
                operation::Instruction::JMP
                    | operation::Instruction::JSR
                    | operation::Instruction::RTS
                    | operation::Instruction::BEQ
                    | operation::Instruction::BCS
                    | operation::Instruction::BNE
                    | operation::Instruction::BCC
            ) {
                self.program_counter += instruction.instruction_byte_length as u16;
            }

            self.instructions_executed += 1;
            if self.cmdline_args.instrumentation && (self.instructions_executed % 10000000 == 0) {
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

    fn adc(&mut self, mode: operation::AddressingMode) {
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
        self.status_flags
            .set_flag(status_reg::Flag::Carry, sum > 255);
        let overflow_flag_after_add: bool = (self.accumulator & (1 << 7)) == 1;
        self.status_flags.set_flag(
            status_reg::Flag::Overflow,
            overflow_flag_after_add != overflow_flag_before_add,
        );
    }
}
