use crate::utils::pause::pause_for_input;
use clap::Parser;
use colored::Colorize;
use crossterm::terminal::disable_raw_mode;
use log::{debug, info};
use std::time::Instant;
use std::{fs, io, usize};
pub mod memory;
pub mod operation;
pub mod status_reg;

use std::io::Write;

use futures::{future::FutureExt, select, StreamExt};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    terminal::enable_raw_mode,
};

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

    // dump info on exit
    #[arg(help = "Dump state on exit", short, long, default_value_t = false)]
    pub dump_state_exit: bool,

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

    // Enable keyboard input
    #[arg(
        help = "Enable keyboard interaction",
        short,
        long,
        default_value_t = false
    )]
    pub keyboard: bool,
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

fn bcd_to_u8(byte: u8) -> Option<u8> {
    let low_nibble = byte & 0xF;
    let high_nibble = (byte >> 4) & 0xF;
    if low_nibble > 9 || high_nibble > 9 {
        None
    } else {
        Some(high_nibble * 10 + low_nibble)
    }
}

impl Cpu6502 {
    pub fn print_state(&mut self) {
        if self.cmdline_args.no_print {
            return;
        }
        self.status_flags.print_status_flags_readable();
        println!("X  = 0x{:#>02x}, {}", self.x_index, self.x_index);
        println!("Y  = 0x{:#>02x}, {}", self.y_index, self.y_index);
        println!("A  = 0x{:#>02x}, {}", self.accumulator, self.accumulator);
        println!("{} = 0x{:#>04x}", "PC".blue(), self.program_counter);
        println!("{} = 0x{:#>02x}", "SP".yellow(), self.stack_pointer);
        self.memory.dump_memory(
            self.cmdline_args.print_all_mem,
            self.program_counter,
            self.stack_pointer,
        );
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
            "\nNEXT INSTRUCTION: {} {}",
            format!("{:?}", instruction.instruction_type).green(),
            operand
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

    pub fn set_byte_wrap(&mut self, index: usize, val: u8) {
        self.memory.set_byte(index, val);
        if self.cmdline_args.keyboard && index == memory::MemMap::CHROUT as usize {
            std::io::stdout().flush().ok().expect("Could not flush :(");
        }
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
        self.set_byte_wrap(addr, self.x_index);
    }

    fn sty(&mut self, mode: operation::AddressingMode) {
        // store index y into memory
        let addr = self.get_addr(mode);
        self.set_byte_wrap(addr, self.y_index);
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
        self.x_index = self.x_index.wrapping_sub(1);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.x_index & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.x_index == 0);
    }

    fn dey(&mut self) {
        self.y_index = self.y_index.wrapping_sub(1);
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
        self.set_byte_wrap(addr, self.accumulator);
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
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if !self.status_flags.z {
            let new_pc = self
                .program_counter
                .wrapping_add_signed(2)
                .wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn bvs(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if self.status_flags.v {
            let new_pc = self
                .program_counter
                .wrapping_add_signed(2)
                .wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn bvc(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if !self.status_flags.v {
            let new_pc = self
                .program_counter
                .wrapping_add_signed(2)
                .wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn bpl(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if !self.status_flags.n {
            let new_pc = self
                .program_counter
                .wrapping_add_signed(2)
                .wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
            self.program_counter += 2;
        }
    }

    fn bmi(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let offset = self.memory.get_byte(addr) as i8;
        if self.status_flags.n {
            let new_pc = self
                .program_counter
                .wrapping_add_signed(2)
                .wrapping_add_signed(offset as i16);
            self.program_counter = new_pc;
        } else {
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

    fn eor(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator = self.accumulator ^ self.memory.get_byte(addr);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, (self.accumulator & 1 << 7) != 0);
    }

    fn inc(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let value = self.memory.get_byte(addr);
        let new_value = value.wrapping_add(1);
        self.set_byte_wrap(addr, new_value);

        self.status_flags
            .set_flag(status_reg::Flag::Zero, new_value == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, new_value & 0b10000000 != 0);
    }

    fn inx(&mut self) {
        self.x_index = self.x_index.wrapping_add(1);

        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.x_index & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.x_index == 0);
    }

    fn iny(&mut self) {
        self.y_index = self.y_index.wrapping_add(1);

        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.y_index & 0b10000000 != 0);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.y_index == 0);
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

    fn pha(&mut self) {
        self.push_stack(self.accumulator);
    }

    fn pla(&mut self) {
        self.accumulator = self.pop_stack();
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.accumulator & 0b10000000 != 0,
        );
    }

    fn plp(&mut self) {
        let new_status = self.pop_stack();
        self.status_flags
            .set_flag(status_reg::Flag::Negative, (new_status & 1 << 7) != 0);

        self.status_flags
            .set_flag(status_reg::Flag::Overflow, (new_status & 1 << 6) != 0);

        self.status_flags
            .set_flag(status_reg::Flag::DecimalMode, (new_status & 1 << 3) != 0);

        self.status_flags
            .set_flag(status_reg::Flag::Interrupt, (new_status & 1 << 2) != 0);

        self.status_flags
            .set_flag(status_reg::Flag::Zero, (new_status & 1 << 1) != 0);

        self.status_flags
            .set_flag(status_reg::Flag::Carry, (new_status & 1) != 0);
    }

    fn tax(&mut self) {
        self.x_index = self.accumulator;
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.x_index == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.x_index & 0b10000000 != 0);
    }

    fn tay(&mut self) {
        self.y_index = self.accumulator;
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.y_index == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, self.y_index & 0b10000000 != 0);
    }

    fn lsr(&mut self, mode: operation::AddressingMode) {
        match mode {
            operation::AddressingMode::Accumulator => {
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, (self.accumulator & 0b1) == 1);
                self.accumulator = self.accumulator >> 1;
                self.status_flags
                    .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
            }
            _ => {
                let addr = self.get_addr(mode);
                let mut val = self.memory.get_byte(addr);
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, (val & 0b1) == 1);
                val = val >> 1;
                self.set_byte_wrap(addr, val);
                self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
            }
        };
        self.status_flags
            .set_flag(status_reg::Flag::Negative, false);
    }

    fn and(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator = self.memory.get_byte(addr) & self.accumulator;
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags
            .set_flag(status_reg::Flag::Negative, (self.accumulator & 1 << 7) != 0);
    }

    fn bit(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        let and_result = self.memory.get_byte(addr) & self.accumulator;
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            (self.memory.get_byte(addr) & 1 << 7) != 0,
        );
        self.status_flags.set_flag(
            status_reg::Flag::Overflow,
            (self.memory.get_byte(addr) & 1 << 6) != 0,
        );
        self.status_flags
            .set_flag(status_reg::Flag::Zero, and_result == 0);
    }

    fn asl(&mut self, mode: operation::AddressingMode) {
        match mode {
            operation::AddressingMode::Accumulator => {
                let shift_out_bit = (self.accumulator & 0b10000000) >> 7;
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, shift_out_bit == 1);
                self.accumulator = self.accumulator << 1;
                let result_msb = (self.accumulator & 0b10000000) >> 7;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, result_msb == 1);
                self.status_flags
                    .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
            }
            _ => {
                let addr = self.get_addr(mode);
                let mut val = self.memory.get_byte(addr);
                let shift_out_bit = (val & 0b10000000) >> 7;
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, shift_out_bit == 1);
                val = val << 1;
                let result_msb = (val & 0b10000000) >> 7;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, result_msb == 1);
                self.set_byte_wrap(addr, val);
                self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
            }
        };
    }

    fn ror(&mut self, mode: operation::AddressingMode) {
        match mode {
            operation::AddressingMode::Accumulator => {
                let carry_before_shift = self.status_flags.c as u8;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, carry_before_shift == 1);
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, (self.accumulator & 0b1) == 1);
                self.accumulator = self.accumulator >> 1;
                self.accumulator |= carry_before_shift << 7;
                self.status_flags
                    .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
            }
            _ => {
                let addr = self.get_addr(mode);
                let mut val = self.memory.get_byte(addr);
                let carry_before_shift = self.status_flags.c as u8;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, carry_before_shift == 1);
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, (val & 0b1) == 1);
                val = val >> 1;
                val |= carry_before_shift << 7;
                self.set_byte_wrap(addr, val);
                self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
            }
        };
    }

    fn brk(&mut self, mode: operation::AddressingMode) {
        todo!("Need to implement brk");
    }

    fn ora(&mut self, mode: operation::AddressingMode) {
        let addr = self.get_addr(mode);
        self.accumulator |= self.memory.get_byte(addr);
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.accumulator & 0b10000000 != 0,
        );
    }

    fn rti(&mut self, mode: operation::AddressingMode) {
        todo!("Need to implement rti");
    }

    fn rol(&mut self, mode: operation::AddressingMode) {
        match mode {
            operation::AddressingMode::Accumulator => {
                let carry_before_shift = self.status_flags.c as u8;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, carry_before_shift == 1);
                self.status_flags.set_flag(
                    status_reg::Flag::Carry,
                    (self.accumulator & 0b10000000) >> 7 == 1,
                );
                self.accumulator = self.accumulator << 1;
                self.accumulator |= carry_before_shift;
                self.status_flags
                    .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
            }
            _ => {
                let addr = self.get_addr(mode);
                let mut val = self.memory.get_byte(addr);
                let carry_before_shift = self.status_flags.c as u8;
                self.status_flags
                    .set_flag(status_reg::Flag::Negative, carry_before_shift == 1);
                self.status_flags
                    .set_flag(status_reg::Flag::Carry, (val & 0b10000000) >> 7 == 1);
                val = val << 1;
                val |= carry_before_shift;
                self.set_byte_wrap(addr, val);
                self.status_flags.set_flag(status_reg::Flag::Zero, val == 0);
            }
        };
    }

    fn php(&mut self) {
        let reg = self.status_flags.as_u8();
        self.push_stack(reg);
    }

    fn push_stack(&mut self, value: u8) {
        let stack_addr = 0x0100 | (self.stack_pointer as u16);
        self.set_byte_wrap(stack_addr as usize, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_addr = 0x100 | (self.stack_pointer as u16);
        return self.memory.get_byte(stack_addr as usize);
    }

    pub fn handle_keyboard(&mut self, reader: &mut EventStream) -> bool {
        let mut event = reader.next().fuse();
        select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event{
                            // println!("{:?},{:?}",key_event.code,key_event.modifiers);
                            match key_event.code {
                                KeyCode::Backspace => {
                                    self.set_byte_wrap(memory::MemMap::CHRIN as usize, 0x08);
                                }
                                KeyCode::Enter => {
                                    self.set_byte_wrap(memory::MemMap::CHRIN as usize, 0x0d);
                                }
                                KeyCode::Char(c) => {
                                    self.set_byte_wrap(memory::MemMap::CHRIN as usize, c as u8);
                                }
                                _ => {}
                            }
                            if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL {
                                println!("Got Ctrl+c getting out of here");
                                return false;
                            }
                            return true;
                        }
                    }
                    Some(Err(_)) => return false,
                    None => return true,
                }
            }
            default => return true,
        };
        return true;
    }

    pub fn run(&mut self) {
        let rvec: u16 =
            (self.memory.get_byte(0xfffd) as u16) << 8 | self.memory.get_byte(0xfffc) as u16;
        self.program_counter = rvec;

        let mut reader = EventStream::new();

        loop {
            if self.cmdline_args.keyboard {
                let success = self.handle_keyboard(&mut reader);
                if !success {
                    println!("Disabled Raw mode and exiting");
                    return;
                }
            }

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
                operation::Instruction::SBC => self.sbc(instruction.mode),
                operation::Instruction::STX => self.stx(instruction.mode),
                operation::Instruction::STY => self.sty(instruction.mode),
                operation::Instruction::LDX => self.ldx(instruction.mode),
                operation::Instruction::LDY => self.ldy(instruction.mode),
                operation::Instruction::CPX => self.cpx(instruction.mode),
                operation::Instruction::CPY => self.cpy(instruction.mode),
                operation::Instruction::DEC => self.dec(instruction.mode),
                operation::Instruction::DEX => self.dex(),
                operation::Instruction::DEY => self.dey(),
                operation::Instruction::EOR => self.eor(instruction.mode),
                operation::Instruction::JMP => self.jmp(instruction.mode),
                operation::Instruction::NOP => {}
                operation::Instruction::LDA => self.lda(instruction.mode),
                operation::Instruction::STA => self.sta(instruction.mode),
                operation::Instruction::JSR => self.jsr(instruction.mode),
                operation::Instruction::RTS => self.rts(),
                operation::Instruction::CMP => self.cmp(instruction.mode),
                operation::Instruction::BPL => self.bpl(instruction.mode),
                operation::Instruction::BEQ => self.beq(instruction.mode),
                operation::Instruction::BNE => self.bne(instruction.mode),
                operation::Instruction::BMI => self.bmi(instruction.mode),
                operation::Instruction::BVS => self.bvs(instruction.mode),
                operation::Instruction::BVC => self.bvc(instruction.mode),
                operation::Instruction::INC => self.inc(instruction.mode),
                operation::Instruction::INX => self.inx(),
                operation::Instruction::INY => self.iny(),
                operation::Instruction::TXS => self.txs(),
                operation::Instruction::TSX => self.tsx(),
                operation::Instruction::TXA => self.txa(),
                operation::Instruction::TYA => self.tya(),
                operation::Instruction::PHA => self.pha(),
                operation::Instruction::PLA => self.pla(),
                operation::Instruction::PLP => self.plp(),
                operation::Instruction::BCS => self.bcs(instruction.mode),
                operation::Instruction::BCC => self.bcc(instruction.mode),
                operation::Instruction::TAX => self.tax(),
                operation::Instruction::TAY => self.tay(),
                operation::Instruction::LSR => self.lsr(instruction.mode),
                operation::Instruction::ROR => self.ror(instruction.mode),
                operation::Instruction::ROL => self.rol(instruction.mode),
                operation::Instruction::PHP => self.php(),
                operation::Instruction::ASL => self.asl(instruction.mode),
                operation::Instruction::AND => self.and(instruction.mode),
                operation::Instruction::BIT => self.bit(instruction.mode),
                operation::Instruction::BRK => self.brk(instruction.mode),
                operation::Instruction::ORA => self.ora(instruction.mode),
                operation::Instruction::RTI => self.rti(instruction.mode),
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
                    | operation::Instruction::BVC
                    | operation::Instruction::BVS
                    | operation::Instruction::BPL
                    | operation::Instruction::BMI
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
                println!(
                    "Have emulated {} instructions so far",
                    self.instructions_executed
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
    fn sbc(&mut self, mode: operation::AddressingMode) {
        // TODO: Decimal mode if status register
        // has decimal mode flag set need to treat
        // hex as decimal for example 0x65 == 65
        let addr = self.get_addr(mode);
        debug!("Address being used to SBC {:#>04x}", addr);
        let carry_subtract = 1 - self.status_flags.c as u8; // Subtract carry (1's complement of carry)
        let mem_val: u16 = self.memory.get_byte(addr as usize) as u16;
        let overflow_flag_before_sub: bool = (self.accumulator & (1 << 7)) == 1;
        let diff = (self.accumulator as u16)
            .wrapping_sub(mem_val)
            .wrapping_sub(carry_subtract as u16);
        self.accumulator = diff as u8;
        self.status_flags
            .set_flag(status_reg::Flag::Carry, diff < 0x100);
        let overflow_flag_after_sub: bool = (self.accumulator & (1 << 7)) == 1;
        self.status_flags.set_flag(
            status_reg::Flag::Overflow,
            overflow_flag_after_sub != overflow_flag_before_sub,
        );
        self.status_flags
            .set_flag(status_reg::Flag::Zero, self.accumulator == 0);
        self.status_flags.set_flag(
            status_reg::Flag::Negative,
            self.accumulator & 0b10000000 != 0,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu6502::bcd_to_u8;

    #[test]
    fn test_bcd_to_u8_valid_input() {
        // Testing valid BCD inputs
        assert_eq!(bcd_to_u8(0x11), Some(11), "0x11 should convert to 11");
        assert_eq!(bcd_to_u8(0x21), Some(21), "0x21 should convert to 21");
        assert_eq!(bcd_to_u8(0x33), Some(33), "0x33 should convert to 33");
        assert_eq!(bcd_to_u8(0x45), Some(45), "0x45 should convert to 45");
        assert_eq!(bcd_to_u8(0x18), Some(18), "0x18 should convert to 18");
        assert_eq!(bcd_to_u8(0x31), Some(31), "0x31 should convert to 31");
        assert_eq!(bcd_to_u8(0x19), Some(19), "0x19 should convert to 19");
        assert_eq!(bcd_to_u8(0x99), Some(99), "0x99 should convert to 99");
    }

    #[test]
    fn test_bcd_to_u8_invalid_input() {
        // Testing an invalid BCD input
        // Assuming 0xF1 is not a valid BCD and should return None
        assert!(
            bcd_to_u8(0xF1).is_none(),
            "0xF1 is not a valid BCD and should return None"
        );

        assert!(
            bcd_to_u8(0xF5).is_none(),
            "0xF5 is not a valid BCD and should return None"
        );

        assert!(
            bcd_to_u8(0x0F).is_none(),
            "0x0F is not a valid BCD and should return None"
        );
    }
}
