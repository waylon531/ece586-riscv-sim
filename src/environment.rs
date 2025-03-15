use clap::builder::Str;
use termion::raw::IntoRawMode;

use crate::machine::ExecutionError;
use std::time::Instant;
use std::io::{stdout,IsTerminal, Write};

/* This is the stupidest way on the planet to do this */
pub fn whichNewLine() -> &'static str {
    match stdout().is_terminal() { true => "\r\n", false => "\n" }
}
pub fn writeNewline() {
    writeStdout(match stdout().is_terminal() { true => "\r\n", false => "\n" });
}
pub fn clearTerm() {
    if (stdout().is_terminal()) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout,"{}",termion::clear::All);
    }
}
pub fn writeStdout(output: &str) {
    if (stdout().is_terminal()) {
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout,"{}",output);
    } else {
        print!("{}", output);
    }

}

pub struct Environment {
    fdtable: [i32; 1024],
    timer: Instant
}

impl Environment {
    pub fn new() -> Self {
        let mut e = Environment {
            fdtable: [0;1024],
            timer: Instant::now()
        };
        e
    }
    pub fn reset_timer(&mut self) -> () {
        self.timer = Instant::now();
    }
    pub fn syscall(&mut self, a7: u32, a0: u32, a1: u32, a2: u32) -> Result<u32, ExecutionError> {
        match a7 {
            // open file descriptor
            62 => { 
                Ok(0)
            },
            // read from file descriptor
            63 => {
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
                Ok(self.timer.elapsed().as_millis() as u32)
            }
            // exit
            94 => {
                std::process::exit(a0 as i32);
            },
            _ => { Err(ExecutionError::InvalidSyscall(a7)) }
        }
    }
}