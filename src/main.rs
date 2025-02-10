mod decode;
mod machine;
mod opcode;
mod register;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    single_step: bool,
    #[arg(value_name = "FILE", default_value = "program.mem")]
    filename: String,
    #[arg(short = 'a', long, default_value_t = 0)]
    starting_addr: u32,
    #[arg(short = 's', long, default_value_t = 65536)]
    stack_addr: u32,
}

fn main() {
    let _cli = Cli::parse();
    println!("Hello, world!");
}
