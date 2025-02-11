mod decode;
mod machine;
mod opcode;
mod register;
mod webui;

use register::Register;
use machine::{Machine,ExecutionError};

use termion::input::TermRead;
use termion::raw::IntoRawMode;

use clap::Parser;
use std::io::{stdin, stdout, Write, Stdin};
use std::process::ExitCode;
use std::thread;

#[derive(Parser, Debug)]
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
    #[arg(short = 'm', long, default_value_t = 64*1024)]
    memory_top: u32,
    #[arg(short = 'W')]
    web_ui: bool,

}

fn main() -> std::io::Result<ExitCode> {
    let cli = Cli::parse();
    // if we're not running the web ui
    if !cli.web_ui {
        // just launch into the simulator
        return run_simulator(cli);
    }
    // otherwise, run simulator and web server in separate threads
    let simulator_thread = thread::spawn(|| {
        run_simulator(cli);
    });
    let web_server_thread = thread::spawn(|| {
        webui::run_server();
    });
    web_server_thread.join().unwrap();
    simulator_thread.join().unwrap();
    // TODO: replace this with exit code of simulator
    return Ok(ExitCode::from(0));
}

fn run_simulator(cli: Cli) -> std::io::Result<ExitCode> {


    // Set up input and output
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let capacity = if cli.memory_top == 0 { 4*1024*1024*1024 } else { cli.memory_top  as usize} ;
    let mmap = vec![0; capacity].into_boxed_slice();

    // TODO: set up machine mmap in a real way instead of this jank

    let mut machine = Machine::new(cli.starting_addr, cli.stack_addr, cli.memory_top, mmap);
    let store_a0_42 = 0b0010011 | (Register::A0.to_num() << 7) | (42 << 20);
    let _ = machine.store_word(store_a0_42 as u32,0);
    // JALR to RA
    let ret = 0b1100111 | (Register::RA.to_num() << 15) ;
    let _ = machine.store_word(ret as u32,4);


    // Either run the machine in single-step mode or all at once
    // maybe TODO: Move this out to another function so we can do better error handling
    // it kind of doesnt matter though
    if cli.single_step {
        loop {
            match machine.step() {
                Ok(()) => {},
                Err(ExecutionError::FinishedExecution(code)) => {
                    return Ok(ExitCode::from(code))
                }
                // If we hit a breakpoint, because we are single-stepping, it is only worth
                // printing an additional message
                Err(e@ ExecutionError::Breakpoint(_)) => {
                    write!(stdout,"{}",e)?;
                },
                // Otherwise all errors are fatal
                Err(e) => {
                    eprintln!("{}",e);
                    return Ok(ExitCode::from(1));
                }
            };
            write!(stdout,"{}",termion::clear::All)?;
            write!(stdout,"{}",machine.display_info())?;
            wait_for_keypress(&stdin);
        }
    } else {
        loop {
            match machine.step() {
                Ok(()) => {},
                Err(ExecutionError::FinishedExecution(code)) => {
                    return Ok(ExitCode::from(code))
                }
                // If we hit a breakpoint then pause execution and wait for a keypress
                Err(e@ ExecutionError::Breakpoint(_)) => {
                    write!(stdout,"{}",termion::clear::All)?;
                    write!(stdout,"{}",machine.display_info())?;
                    write!(stdout,"\n{}",e)?;
                    wait_for_keypress(&stdin);
                },
                // Otherwise all errors are fatal
                Err(e) => {
                    eprintln!("{}",e);
                    return Ok(ExitCode::from(1));
                }
            }

        }

    }
    // Note: The loop will return right now and never fall through to this
    // Ok(ExitCode::SUCCESS)
}

// Read a single keypress
// NOTE: throws away errors silently
fn wait_for_keypress<>(stdin: &Stdin) {
    let _ = stdin.lock().keys().next();
}
