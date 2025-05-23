mod api;
mod debugger;
mod decode;
#[allow(dead_code)]
mod devices;
#[allow(dead_code)]
mod environment;
mod machine;
mod opcode;
mod register;
#[allow(dead_code)]
mod statetransfer;

use machine::{ExecutionError, Machine};
use devices::DeviceConfig;

use thiserror::Error;

use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{stdout, stdin, Write, BufReader, BufRead, IsTerminal};
use std::num;
use std::process::ExitCode;
use std::thread;

use single_value_channel::{channel_starting_with,Updater as SvcSender};
use crossbeam_channel::{unbounded,Receiver as CbReceiver, Sender as CbSender};
use statetransfer::{MachineState};

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum DumpFmt {
    JSON,
    Txt,
}

// TODO: memory top maybe could be a string? For 1GB? Etc
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, alias="silent")]
    quiet: bool,
    #[arg(long)]
    single_step: bool,
    #[arg(value_name = "FILE", default_value = "program.mem")]
    filename: String,
    #[arg(short = 'a', long, default_value_t = 0)]
    starting_addr: u32,
    #[arg(short = 's', long)]
    stack_addr: Option<u32>,
    #[arg(short = 'm', long, default_value_t = 64*1024)]
    memory_top: u32,
    #[arg(short = 'W')]
    web_ui: bool,

    /// Dump machine state to filename DUMP_TO when finished
    #[arg(short, long)]
    dump_to: Option<String>,

    #[arg(long, value_enum, default_value_t = DumpFmt::Txt)]
    dump_fmt: DumpFmt,

    /// Suppress exit code returned from emulated program
    #[arg(long)]
    suppress_status: bool,

    // These have to be parsed later, clap isnt smart enough to parse them
    /// Enable a specific device. Format is `--device NAME,opt=foo,opt2=foo2`.
    #[arg(long)]
    device: Vec<String>
}

fn main() -> std::io::Result<ExitCode> {
    let cli = Cli::parse();
    if cli.single_step && ! stdout().is_terminal() {
        println!("Cannot enter interactive mode when stdout is not a terminal.");
        return Ok(ExitCode::FAILURE);
    }
    // if we're not running the web ui
    if !cli.web_ui {
        // just launch into the simulator
        return run_simulator(cli, None, None);
    }
    // otherwise, run simulator and web server in separate threads
    
    // Create an unbounded channel for control messages from the web UI to the machine
    /* 
    
    TODO: create data type for machine state and commands
    
     */
    let (commands_tx, commands_rx): (CbSender<statetransfer::ControlCode>, CbReceiver<statetransfer::ControlCode>) = unbounded();
    
    // Create a channel to send the machine state through to the web UI 
    let (state_rx, state_tx) = channel_starting_with(MachineState::empty());
    
    let simulator_thread = thread::spawn(|| {
        let cli_for_simulator = cli; // Move cli into a new variable
        let _ = thread::spawn(move || {
            run_simulator(cli_for_simulator, Some(commands_rx), Some(state_tx)).unwrap();
        });
    });
    let web_server_thread = thread::spawn(|| {
        api::run_server(commands_tx, state_rx);
    });
    web_server_thread.join().unwrap();
    simulator_thread.join().unwrap();
    // TODO: replace this with exit code of simulator
    return Ok(ExitCode::from(0));
}

fn run_simulator(cli: Cli, commands_rx: Option<CbReceiver<statetransfer::ControlCode>>, state_tx: Option<SvcSender<statetransfer::MachineState>>) -> std::io::Result<ExitCode> {
    // Set up input and output
    
    
    let stdin = stdin();

    let capacity = if cli.memory_top == 0 {
        4 * 1024 * 1024 * 1024
    } else {
        cli.memory_top as usize
    };

    // Initialize extra hw devices
    let mut devices = Vec::new();
    for device_str in cli.device {
        match DeviceConfig::from_str(&device_str) {
            Ok(config) => match config.into_device() {
                Ok(device) => devices.push(device),
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(ExitCode::FAILURE);
                },
            },
            Err(e) => {
                eprintln!("{}", e);
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    // Check to make sure we can open dump_to and overwrite it
    // In case of a crash this file will then be empty
    let dump_to = match cli.dump_to {
        Some(f) => Some(File::create(f)?),
        None => None,
    };

    let mut mmap = vec![0; capacity];

    // TODO: set up machine mmap in a real way instead of this jank
    match parse_file(&mut mmap, &cli.filename) {
        Ok(()) => {}
        Err(ReadFileError::IoError(e)) => return Err(e),
        Err(e) => {
            eprintln!("{}", e);
            return Ok(ExitCode::FAILURE);
        }
    }

    let mut machine = Machine::new(
        cli.starting_addr,
        cli.stack_addr,
        cli.memory_top,
        mmap.into_boxed_slice(),
        cli.verbose,
        devices
    );

    // Run the machine to completion
    let result = machine.run(cli.single_step, &stdin, commands_rx,state_tx);
    let mut error_message = None;

    let status_code = match result {
        // This should legitimately be unreachable, Ok(()) is never returned anywhere
        Ok(()) => unreachable!(),
        Err(ExecutionError::FinishedExecution(code)) => Ok(ExitCode::from(code)),
        Err(e) => {
            error_message = Some(format!("{}",e));
            Err(ExitCode::from(1))
        }
    };

    // Either run the machine in single-step mode or all at once
    // Handle all cleanup/finishing actions
    if let Some(mut file) = dump_to {
        let bytes = match cli.dump_fmt {
            DumpFmt::JSON => serde_json::to_string(&machine)?,
            DumpFmt::Txt => machine.dump_state_txt(),
        };
        // Note: if this fails it will override the status code spit out by the child program
        file.write_all(bytes.as_bytes())?;
    }

    // Print the registers one last time
    // this is silly, but to explain: in normal mode, it prints a real-time display. in quiet mode, it prints nothing. in verbose mode, it dumps in a basic format at every instruction.
    if !(cli.quiet|cli.verbose) {

        environment::clear_term();
        environment::write_stdout(&machine.display_info());

    }

    if let Some(err) = error_message {
        environment::write_newline();
        environment::write_stdout(&err);
        environment::write_newline();
    }

    // Exit
    // Determine whether to throw away the status code or not
    match (status_code, cli.suppress_status) {
        (Ok(_), true) => Ok(ExitCode::SUCCESS),
        (Err(_), true) => Ok(ExitCode::FAILURE),
        (Ok(s), false) => Ok(s),
        (Err(s), false) => Ok(s),
    }
}


fn parse_file(bytes: &mut Vec<u8>, filename: &str) -> Result<(), ReadFileError> {
    let f = File::open(filename)?;
    let reader = BufReader::new(f);
    for line in reader.lines() {
        let line = line?;
        let (addr, data) = (&line)
            .split_once(":")
            .ok_or(ReadFileError::ParseError(line.clone()))?;
        let addr: usize = u32::from_str_radix(addr.trim(), 16)? as usize;
        // TODO: can have byte and word strings
        // look for number of characters
        let mut offset = 0;
        let data_chunks = data.trim().split(' ');
        for data in data_chunks {
            let len = data.len();
            let data = u32::from_str_radix(data.trim(), 16)?;
            bytes[addr+offset] = data as u8;
            if len >= 4 {
                bytes[addr + offset + 1 as usize] = (data >> 8) as u8;
            }
            if len >= 8 {
                bytes[addr + offset + 2 as usize] = (data >> 16) as u8;
                bytes[addr + offset + 3 as usize] = (data >> 24) as u8;
            }
            offset += len/2;
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
    ParseIntError(#[from] num::ParseIntError),
}
