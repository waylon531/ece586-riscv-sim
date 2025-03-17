use clap::builder::Str;
use filedescriptor::FileDescriptorTable;
use termion::raw::IntoRawMode;

use crate::machine::ExecutionError;
use std::fs::File;
use std::{mem, str};
use std::time::Instant;
use std::io::{stdout,IsTerminal, Write};
mod filedescriptor;

/* This is the stupidest way on the planet to do this */
pub fn which_new_line() -> &'static str {
    match stdout().is_terminal() { true => "\r\n", false => "\n" }
}
pub fn write_newline() {
    write_stdout(match stdout().is_terminal() { true => "\r\n", false => "\n" });
}
pub fn clear_term() {
    if (stdout().is_terminal()) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout,"{}",termion::clear::All);
    }
}
pub fn write_stdout(output: &str) {
    if (stdout().is_terminal()) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout,"{}",output);
    } else {
        print!("{}", output);
    }

}

pub struct Environment {
    fdtable: FileDescriptorTable,
    timer: Instant
}

impl Environment {
    pub fn new() -> Self {
        let mut e = Environment {
            fdtable: FileDescriptorTable::new(),
            timer: Instant::now()
        };
        e
    }
    pub fn reset_timer(&mut self) -> () {
        self.timer = Instant::now();
    }
    pub fn syscall(&mut self, a7: u32, a0: u32, a1: u32, a2: u32, memory: &mut Box<[u8]>) -> Result<i32, ExecutionError> {

        let mut read_bytes = |start: u32, len:u32 | -> Vec<u8> { memory[start as usize..len as usize].to_vec() };
        let mut read_string = |start:u32| -> Vec<u8> { memory[start as usize..(memory[start as usize..].iter().position(|&c| -> bool { c==b'\0' }).unwrap_or(memory.len() - start as usize))].to_vec() };
        match a7 {
            // open(char* pathname, int flags)
            // a0: pathname, a1: flags
            // returns file descriptor to a0, or -1 if failed 
            56 => {
                let filename = read_string(a0);
                Ok(self.fdtable.open(&filename, a1).unwrap_or_else(|_|{
                    eprintln!("ENVIRONMENT: Failed to open file: {}",str::from_utf8(&filename).unwrap_or("[garbled nonsense]"));
                    -1
                }))
                //self.fdtable.open(a1)
            },
            // read from file descriptor
            // a0: fd, a1: start address, a2: max length to store
            63 => {
                /* TODO: Implement this */
                Ok(0)
                /*
                let mut f = unsafe { File::from_raw_fd(*a0 as i32) };
                let mut buf = vec![0;a2 as usize];
                
                
                match f.read(&mut buf) {
                Ok(bytes_read) => { bytes_read/ }
                Err(_) => { self.set_reg(Register::A0, (0-1) as u32); }
                }
                */
            }
            // write to file descriptor
            64 => {
                Ok(0)
                /* let mut f = unsafe { File::from_raw_fd(a0 as i32) }; */
                // literally no idea if this will work
                /*match write!(&mut f, "Hello, world!") {
                //Ok(bytes_written) => { self.set_reg(Register::A0, bytes_written as u32); },
                Ok(()) => {}
                Err(_) => { self.set_reg(Register::A0, (0-1) as u32); }
                }*/
            },
            // sleep
            77 => {
                std::thread::sleep(std::time::Duration::from_millis(a0 as u64));
                Ok(0)
            },
            // return time elapsed
            78 => {
                Ok(self.timer.elapsed().as_millis() as i32)
            }
            // exit
            94 => {
                std::process::exit(a0 as i32);
            },
            _ => { Err(ExecutionError::InvalidSyscall(a7)) }
        }
    }
}