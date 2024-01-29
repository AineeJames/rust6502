use clap::Parser;
pub mod cpu6502;

mod utils {
    pub mod pause;
}

fn main() {
    let args: cpu6502::Args = cpu6502::Args::parse();
    env_logger::init();

    println!("Running {}!", args.binary_file);
    println!("Printing all mem {}!", args.print_all_mem);

    let mut cpu: cpu6502::Cpu6502 = cpu6502::init_cpu6502(args);

    cpu.load_file_into_memory();
    cpu.run();
}
