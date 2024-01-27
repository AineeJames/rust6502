use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // 6502 hex file to run
    #[arg(short, long)]
    binary_file: String,

    // Print all mem even if zeroed
    #[arg(short, long, default_value_t = true)]
    print_all_mem: bool,
}

const MEM_SIZE: usize = 65536;

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
enum Flag{
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
   c: bool
}
impl StatusFlags{
    fn print_status_flags_readable(&self){
        println!("Negative Flag: {}", self.n);
        println!("Overflow Flag: {}", self.o);
        println!("Unused Flag: {}", self.u);
        println!("Break Flag: {}", self.b);
        println!("Decimal Mode: {}", self.d);
        println!("Interrupt Disable: {}", self.i);
        println!("Zero Flag: {}", self.z);
        println!("Carry Flag: {}\n", self.c);    
    }
    fn set_flag(&mut self,  to_set_flag: Flag, flag_state: bool){
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
    status_flags: StatusFlags    
}


fn init_cpu6502() -> Cpu6502 {
    let mut cpu = Cpu6502 {
        memory : vec![0; MEM_SIZE], // stack (0x0100, 0x01FF)
        accumulator : 0,
        x_index : 0,
        y_index : 0,
        program_counter : 0,
        stack_pointer : 0,
        status_flags : StatusFlags{
            n : false,
            o : false,
            u : true,
            b : false,
            d : false,
            i : false,
            z : false,
            c : false,
       }
    };
    // reset vec
    cpu.memory[0xfffc] = 0x06;
    cpu.memory[0xfffd] = 0x00;

    cpu
}

impl Cpu6502{
    fn set_accumulator(&mut self, newacc: u8){
        self.accumulator = newacc;
    }
    
    fn dump_memory(&self) {
        for i in (0..MEM_SIZE).step_by(0x10) {
            let slice = &self.memory[i..i+0x10];

            if slice.iter().any(|&x| x > 0) {
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

    fn print_state(&self){
        self.status_flags.print_status_flags_readable();
        println!("X register = {}",self.x_index);
        println!("Y register = {}",self.y_index);
        println!("Accumulator = {}",self.accumulator);
        println!("Program Counter = {}",self.program_counter);
        println!("Stack Pointer = {}",self.stack_pointer);
    }

    fn load_code_into_memory(&self, code: &Vec<u8>, org: usize) {
        unimplemented!("NOT DONEWARD");
    }
}

fn main() {
    let args = Args::parse();

    println!("Running {}!", args.binary_file);
    println!("Printing all mem {}!", args.print_all_mem);

    // TODO add command line arg to print all memory 
    let mut cpu: Cpu6502 = init_cpu6502();
    cpu.set_accumulator(2);

    cpu.set_accumulator(2);

    cpu.memory[0] = 'a' as u8;
    cpu.memory[1] = 'i' as u8;
    cpu.memory[2] = '/' as u8;
    cpu.memory[3] = 'e' as u8;
    cpu.memory[4] = 'n' as u8;
    cpu.memory[0x0205] = 0x22;

    cpu.dump_memory();
    cpu.print_state();
}