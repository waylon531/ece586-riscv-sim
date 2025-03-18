use clap::builder::Str;
use filedescriptor::FileDescriptorTable;
use termion::raw::IntoRawMode;

use crate::machine::ExecutionError;
use std::fs::File;
use std::os::fd;
use std::{mem, str};
use std::time::Instant;
use std::io::{stdout, IsTerminal, Read, Write};
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
            },
            57 => Ok(self.fdtable.close(a0)),
            // read from file descriptor
            // a0: fd, a1: start address, a2: max length to store
            63 => {
                let mut buf = vec![0;a2 as usize];
                let result = match a0 {
                    // error if trying to read stdout or stderr
                    0 => {
                        Ok(std::io::stdin().read(&mut buf)? as i32)
                    },
                    1|2 => {
                        Ok(-1)                        
                    }
                    _ => {
                        /* TODO: implement flags */
                        let f_idx = self.fdtable.get_idx(a0);
                        if(f_idx<0) { return Ok(-1) };
                        let mut f = self.fdtable.get_file(a0).unwrap();
                        f.read(&mut buf).map(|x| x as i32).map_err(|e| ExecutionError::IOError(e))
                    }
                };
                if(a1+a2 > memory.len() as u32) { return Err(ExecutionError::LoadAccessFault(a1)) };
                // Actually write the file's contents to memory
                memory[a1 as usize..a2 as usize].copy_from_slice(&buf);
                result

            }
            // write to file descriptor
            64 => {
                let mut buf = vec![0;a2 as usize];
                if(a1+a2 > memory.len() as u32) { return Err(ExecutionError::LoadAccessFault(a1)) };
                // read memory into buffer
                buf = memory[a1 as usize..a2 as usize].to_vec();
                let result = match a0 {
                    // error if trying to write to stdin
                    0 => {
                        Ok(-1)
                    },
                    1 => {
                        Ok(std::io::stdout().write(&buf)? as i32)                    
                    },
                    2 => {
                        Ok(std::io::stderr().write(&buf)? as i32)
                    }
                    _ => {
                        /* TODO: implement flags */
                        let f_idx = self.fdtable.get_idx(a0);
                        if(f_idx<0) { return Ok(-1) };
                        let mut f = self.fdtable.get_file(a0).unwrap();
                        f.read(&mut buf).map(|x| x as i32).map_err(|e| ExecutionError::IOError(e))
                    }
                };
                result
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