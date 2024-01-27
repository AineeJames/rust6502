use clap::Parser;
use log::{debug, info};
use std::fs;
use std::io;
use std::io::prelude::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // 6502 hex file to run
    #[arg(short, long)]
    binary_file: String,

    // Print all mem even if zeroed
    #[arg(short, long, default_value_t = false)]
    print_all_mem: bool,

    // Print all mem even if zeroed
    #[arg(short, long, default_value_t = false)]
    step_debug: bool,
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

const MEM_SIZE: usize = 65536;

enum AddressingMode {
    Accumulator,
    Implied,
    Immediate,
    Absolute,
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
    o: bool,
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
        println!("Overflow Flag: {}", self.o);
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
            Flag::Overflow => self.o = flag_state,
            Flag::Unused => self.u = flag_state,
            Flag::Break => self.b = flag_state,
            Flag::DecimalMode => self.d = flag_state,
            Flag::Interrupt => self.i = flag_state,
            Flag::Zero => self.z = flag_state,
            Flag::Carry => self.c = flag_state,
        }
    }
}

struct Cpu6502 {
    memory: Vec<u8>,
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    program_counter: u16,
    stack_pointer: u8,
    status_flags: StatusFlags,
    cmdline_args: Args,
}

fn init_cpu6502(args: Args) -> Cpu6502 {
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
            o: false,
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

impl Cpu6502 {
    fn set_accumulator(&mut self, newacc: u8) {
        self.accumulator = newacc;
    }

    fn dump_memory(&self) {
        for i in (0..MEM_SIZE).step_by(0x10) {
            let slice = &self.memory[i..i + 0x10];

            if slice.iter().any(|&x| x > 0) || self.cmdline_args.print_all_mem {
                print!("0x{i:#>04x}: ");
                for byte in slice.iter().cloned() {
                    print!("{byte:02x} ");
                }

                // aiden char print
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

    fn print_state(&self) {
        self.status_flags.print_status_flags_readable();
        println!("X register = {}", self.x_index);
        println!("Y register = {}", self.y_index);
        println!("Accumulator = 0x{:#>04x}", self.accumulator);
        println!("Program Counter = 0x{:#>04x}", self.program_counter);
        println!("Stack Pointer = {}", self.stack_pointer);
    }

    fn load_file_into_memory(&mut self, org: usize) {
        let code_result: Result<Vec<u8>, std::io::Error> =
            fs::read(self.cmdline_args.binary_file.clone());
        let code = match code_result {
            Ok(code) => code,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
        self.memory[org..org + code.len()].copy_from_slice(&code);
    }

    fn run(&mut self) {
        let rvec: u16 = (self.memory[0xfffc] as u16) << 8 | self.memory[0xfffd] as u16;
        self.program_counter = rvec;
        loop {
            match *self.memory.get(self.program_counter as usize).unwrap() {
                // LDA Immediate
                0xa9 => {
                    info!("LDA");
                    self.accumulator = *self
                        .memory
                        .get((self.program_counter + 1) as usize)
                        .unwrap();
                    self.program_counter += 2;
                }
                // INX
                0xe8 => {
                    info!("INX");
                    self.x_index += 1;
                    self.program_counter += 1;
                }
                // INY
                0xc8 => {
                    info!("INY");
                    self.y_index += 1;
                    self.program_counter += 1;
                }
                // SEC
                0x38 => {
                    info!("SEC");
                    self.status_flags.c = true;
                    self.program_counter += 1;
                }
                // INC
                0xe6 => {
                    // inc memory immediate
                    info!("INC");
                    self.memory[(self.program_counter + 1) as usize] += 1;
                    self.program_counter += 2;
                }
                // NOP
                0xea => {
                    info!("NOP");
                    self.program_counter += 1
                }
                _ => {
                    info!("Unimplemented instruction");
                    println!("Not implemented instruction")
                }
            }
            self.print_state();
            if self.cmdline_args.step_debug {
                pause();
            }
        }
    }
}

fn main() {
    // TODO add nice logging where we can print the instruction name and program counter
    let args = Args::parse();
    env_logger::init();

    println!("Running {}!", args.binary_file);
    println!("Printing all mem {}!", args.print_all_mem);

    // TODO add command line arg to print all memory
    let mut cpu: Cpu6502 = init_cpu6502(args);
    cpu.set_accumulator(2);

    cpu.load_file_into_memory(0x0600);

    cpu.run();

    cpu.dump_memory();
    cpu.print_state();
}
