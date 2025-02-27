mod decode;
mod machine;
mod opcode;
mod register;
mod webui;

use machine::{Machine,ExecutionError};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
use thiserror::Error;

use clap::{ValueEnum,Parser};
use std::fs::File;
use std::io::{stdin, stdout, Write, Stdin, BufReader, BufRead};
use std::num;
use std::process::ExitCode;
use std::thread;

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum DumpFmt {
    JSON,
    Txt
}

// TODO: memory top maybe could be a string? For 1GB? Etc
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
    #[arg(short = 's', long, default_value_t = 64*1024)]
    stack_addr: u32,
    #[arg(short = 'm', long, default_value_t = 64*1024)]
    memory_top: u32,
    #[arg(short = 'W')]
    web_ui: bool,

    /// Dump machine state to filename DUMP_TO when finished
    #[arg(short,long)]
    dump_to: Option<String>,

    #[arg(long, value_enum, default_value_t = DumpFmt::Txt)]
    dump_fmt: DumpFmt,

    /// Suppress exit code returned from emulated program
    #[arg(long)]
    suppress_status: bool,

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
    
    // Check to make sure we can open dump_to and overwrite it
    // In case of a crash this file will then be empty
    let dump_to = match cli.dump_to {
        Some(f) => Some(File::create(f)?),
        None => None
    };

    let mut mmap = vec![0; capacity];

    // TODO: set up machine mmap in a real way instead of this jank
    match parse_file(&mut mmap, &cli.filename) {
        Ok(()) => {},
        Err(ReadFileError::IoError(e)) => return Err(e),
        Err(e) => {
            eprintln!("{}",e);
            return Ok(ExitCode::FAILURE)
        }
    }

    let mut machine = Machine::new(cli.starting_addr, cli.stack_addr, cli.memory_top, mmap.into_boxed_slice());

    // Either run the machine in single-step mode or all at once
    // maybe TODO: Move this out to another function so we can do better error handling
    // it kind of doesnt matter though
    let status_code = if cli.single_step {
        loop {
            match machine.step() {
                Ok(()) => {},
                Err(ExecutionError::FinishedExecution(code)) => {
                    break Ok(ExitCode::from(code));
                }
                // If we hit a breakpoint, because we are single-stepping, it is only worth
                // printing an additional message
                Err(e@ ExecutionError::Breakpoint(_)) => {
                    write!(stdout,"{}",e)?;
                },
                // Otherwise all errors are fatal
                Err(e) => {
                    eprintln!("{}",e);
                    break Err(ExitCode::from(1));
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
                    break Ok(ExitCode::from(code));
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
                    break Err(ExitCode::from(1));
                }
            }

        }

    };
    // Handle all cleanup/finishing actions
    if let Some(mut file) = dump_to {
        let bytes = match cli.dump_fmt {
            DumpFmt::JSON => serde_json::to_string(&machine)?,
            DumpFmt::Txt => machine.dump_state_txt(),
        };
        // Note: this will override the status code spit out by the child program
        file.write_all(bytes.as_bytes())?;
    }
    // Exit
    // Determine whether to throw away the status code or not
    match (status_code,cli.suppress_status) {
        (Ok(_),true) => Ok(ExitCode::SUCCESS),
        (Err(_),true) => Ok(ExitCode::FAILURE),
        (Ok(s),false) => Ok(s),
        (Err(s),false) => Ok(s),

    }
}

// Read a single keypress
// NOTE: throws away errors silently
fn wait_for_keypress<>(stdin: &Stdin) {
    let _ = stdin.lock().keys().next();
}

fn parse_file(bytes: &mut Vec<u8>, filename: &str) -> Result<(),ReadFileError> {
    let f = File::open(filename)?;
    let reader = BufReader::new(f);
    for line in reader.lines() {
    
        let line = line?;
        let (addr, data) = (&line).split_once(":").ok_or(ReadFileError::ParseError(line.clone()))?;
        let addr: usize = u32::from_str_radix(addr.trim(), 16)? as usize;
        // TODO: can have byte and word strings
        // look for number of characters
        let len = data.len();
        let data = u32::from_str_radix(data.trim(), 16)?;
        bytes[addr] = data as u8;
        if len >= 4 {
            bytes[addr + 1 as usize] = (data >> 8) as u8;
        }
        if len >= 8 {
            bytes[addr + 2 as usize] = (data >> 16) as u8;
            bytes[addr + 3 as usize] = (data >> 24) as u8;
        }


    }
    Ok(())

}

#[derive(Error, Debug)]
pub enum ReadFileError {
    #[error("Failed to parse line: {0}")]
    ParseError(String),
    #[error("IO ERROR: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse number: {0}")]
    ParseIntError(#[from] num::ParseIntError)
}
