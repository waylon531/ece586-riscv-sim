mod instruction;
mod stages;
mod alu;
mod memory;
mod regfile;

use crate::debugger::{DebugCommand,self};
/*
use crate::decode::ParseError;
use crate::opcode_old::Operation;
use crate::regfile::Register;
*/
use memory::Memory;
use instruction::Instruction;
use regfile::{RegisterFile,RegABI};
use rustyline::error::ReadlineError;
use serde::Serialize;
use core::time;
use std::fmt::Write;
use std::io::Seek;
use std::thread::sleep;
use std::{
    fs::File,
    io::{self, Read,Write as ioWrite,Stdout,Stdin},
    os::unix::io::FromRawFd,
};
use single_value_channel::Updater as SvcSender;
use crossbeam_channel::Receiver as CbReceiver;
use educe::Educe;

use thiserror::Error;

use crate::statetransfer;
use crate::environment::{self, Environment};


#[derive(Serialize)]
pub struct Machine {
    // Maybe this should be  on the heap
    // How is memory mapped? Is there max 64K? Or is that just the size of program
    // we can load?
    //
    // NOTE: This is a boxed slice, while it could well be a Vec for simplicity
    // we also really don't want to have someone resizing the mmap
    #[serde(skip_serializing)]
    memory: Memory,
    // Store x1-x31
    // x0 is always 0, no reason to store
    registers: RegisterFile,
    // Whether we should step over a breakpoint or not
    pass_breakpoint: bool,
    // NOTE: A vec means we linear search through breakpoints, but also gives a map from 
    //       index/breakpoint number to breakpoint which is nice for ui
    //
    //       Might be way slow though to iterate through this every cycle though
    breakpoints: Vec<u32>,
    #[serde(skip_serializing)]
    env: Environment
}
impl Machine {
    pub fn new(starting_addr: u32, stack_addr: Option<u32>, memory_top: u32, memmap: Box<[u8]>) -> Self{
        let mut m = Machine {
                    memory: Memory::new(memmap, memory_top),
                    registers: RegisterFile::new(),
                    pass_breakpoint: false,
                    breakpoints: Vec::new(),
                    env:Environment::new()
        };
        // set the PC to the start address
        m.registers.pc.write(starting_addr);
        // Set the stack pointer to the lowest invalid memory address by default, aligning down to
        // nearest 16 bytes
        m.registers.from_abi(RegABI::SP).write(stack_addr.unwrap_or(memory_top & !(0xF)));
        m
    }
    /// Run the machine til completion, either running silently until an error is hit or bringing
    /// up the debugger after every step
    pub fn run(&mut self, single_step: bool, _stdin: &Stdin, commands_rx: Option<CbReceiver<i32>>, state_tx: Option<SvcSender<i32>>) -> Result<(),ExecutionError> {
        // reset timer
        self.env.reset_timer();
        /* TODO: Check if commands and state channels are present */


        // NOTE: this cannot be a global include as it conflicts with fmt::Write;
        use std::io::Write;
        let mut should_trigger_cmd = single_step;
        let mut rl = rustyline::DefaultEditor::new()?;
        // Set the default command to step, by default
        let mut last_cmd = DebugCommand::STEP(1);
        let mut should_step = None;
        // Status messages to print
        let mut status: Vec<String> = Vec::new();
        let mut watchlist: Vec<DebugCommand> = Vec::new();
        loop {
            /*
                At the start of each cycle, if the web server is running, we want to communicate with it.
                We exchange information - we send the current state of the machine, and we read commands sent from the web interface.
                Those commands may include modifications to registers or memory addresses - so we interpret those.
             */
            // Check if we stepping for N times, and if we are at the end then pull the debugger
            // back up
            // Otherwise decrement the step counter
            if let Some(count) = should_step {
                if count <= 1 {
                    should_trigger_cmd = true;
                    should_step = None;
                } else {
                    should_step = Some(count-1);
                }
            };

            if should_trigger_cmd {
                // print debug state
                environment::clear_term();
                environment::write_stdout(&self.display_info());
                environment::write_newline();


                // handle all watchlist lines
                // this is way hackier than I thought ...
                for cmd in watchlist.iter() {
                    let mut dummy_run = false;
                    let mut dummy_watchlist = Vec::new();
                    let mut new_status = cmd.execute(
                        self,
                        &mut should_step, 
                        &mut should_trigger_cmd,
                        &mut dummy_run,
                        &mut dummy_watchlist
                        )?;
                    for line in new_status.drain(..) {
                        status.push(line);
                    }
                }

                // handle all watchlist lines
                // this is way hackier than I thought ...
                for cmd in watchlist.iter() {
                    let mut dummy_run = false;
                    let mut dummy_watchlist = Vec::new();
                    let mut new_status = cmd.execute(
                        self,
                        &mut should_step, 
                        &mut should_trigger_cmd,
                        &mut dummy_run,
                        &mut dummy_watchlist
                        )?;
                    for line in new_status.drain(..) {
                        status.push(line);
                    }
                }

                // print status lines
                for line in status.drain(..) {
                    environment::write_stdout(&line);
                    environment::write_newline();
                }

                environment::write_stdout("\r\n");

                // read prompt
                let readline = rl.readline(">> ");
                let read_value = match readline {
                    Ok(line) => line,
                    // Ctrl-C
                    Err(ReadlineError::Interrupted) => {
                        return Err(ExecutionError::HaltedByUser)
                    },
                    // Ctrl-D
                    Err(ReadlineError::Eof) => {
                        return Err(ExecutionError::HaltedByUser)
                    },
                    // If readline throws any other error then bail with the corresponding error
                    // captured
                    Err(e) => return Err(e)?
                };
                // parse and handle debug command
                let command = match read_value.as_str() {
                    "" => last_cmd.clone(),
                    read_value => match DebugCommand::from_string(read_value) {
                        Ok(val) => val,
                        Err(e) => {
                            status.push(format!("{}",e));
                            continue
                        }
                    }
                };

                let mut run = false;
                
                let mut new_status = command.execute(
                    self,
                    &mut should_step, 
                    &mut should_trigger_cmd,
                    &mut run,
                    &mut watchlist
                    )?;


                // Combine vecs
                for line in new_status.drain(..) {
                    status.push(line);
                }

                // Many commands will instantly return control back to the prompt without
                // stepping execution at all
                if !run {
                    continue;
                }

                // Note: this will only get updated on commands that step or continue
                //       not 100% sure that's what we want yet
                last_cmd = command;

            }
            match self.step() {
                Ok(()) => {},
                // Should errors bail? Or bring up the debugger to explore program state?
                // Bail for now probably, its easier (though worse)
                Err(e@ ExecutionError::Breakpoint(_)) => {
                    should_trigger_cmd = true;
                    // Give a pass so the next step of execution can make it past the breakpoint
                    self.pass_breakpoint = true;
                    status.push(format!("{}",e));
                },
                Err(e) => return Err(e)

            }
            // Check if the user is pressing ctrl-c, and if they are, drop back into the debugger
            // oops this needs a bonus thread, this is going to suck
            // the thread can get spun up whenever we are running and spun down, or paused, when we
            // go to the debugger
            
        }
        
    }


    // Fetch, decode, and execute an instruction
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        // First, check if we're at a breakpoint, and cannot pass over it
        if self.breakpoints.contains(& self.registers.pc.read()) && !self.pass_breakpoint {
            return Err(ExecutionError::Breakpoint(self.registers.pc.read()))
        } else {
            // Unset breakpoint pass
            self.pass_breakpoint = false;

        }
        //Fetch and decode
        //let op = Operation::from_bytes(self.read_instruction_bytes(self.pc)?)?;

        // Branches and jumps will set this to false
        let mut increment_pc = true;

        let pc = self.registers.pc.read();

        let op = Instruction::from_bytes(self.memory.read_instruction_bytes(pc));

        stages::execute(opcode, &regfile, &memory);


        if increment_pc {
            self.registers.pc.write(self.registers.pc.value.overflowing_add(4).0);
        }

        
        Ok(())
    }

    // String formatting should never fail, it's likely safe to unwrap here
    pub fn display_info(&self) -> String {
        let mut buf = String::new();
        // Why am I doing this crazy shit? To ensure we only print terminal control characters if the output is a terminal.
        if environment::which_new_line() == "\r\n" { write!(buf,"{}","\r").unwrap(); };
        write!(buf,"PC:\t  {:#010x}", self.pc).unwrap();
        for i in 0 .. 31 {
            write!(buf,"{}",environment::which_new_line()).unwrap();
            write!(buf,"{1:?}:\t{0:>12}\t{0:#010x}",self.registers.from_num(i).unwrap().read(),self.registers.from_num(i).unwrap().abi_name);
            if i < 16 {
                let context: i32 = (i as i32-8)*4;
                let to_fetch = self.registers.pc.value.overflowing_add_signed(context);
                match to_fetch {
                    (addr, false) => {
                        let display_me = match stages::read_instruction_bytes(&self.memory, addr) {
                            Ok(bytes) => match Instruction::from_bytes(bytes) {
                                Ok(op) => format!("{}",op),
                                Err(e) => format!("{}",e)
                            },
                            Err(e) => format!("{}",e)
                        };
                        if i == 8 {
                            write!(buf,"\t PC ->  {addr:#010x}: {}",display_me).unwrap();
                        } else {
                            write!(buf,"\t\t{addr:#010x}: {}",display_me).unwrap();
                        }
                    },
                    (_num, true) => {

                    }
                };
            } else if i > 16 {
                // Offset to fetch
                let context = (i-17)*4;
                let to_fetch = self.registers.from_abi(RegABI::SP).read().saturating_add(context as u32);
                let display_me = match self.memory.read_word(to_fetch) {
                    Ok(word) =>  format!("{0:#010x}",word),
                    Err(e) => format!("{}",e)
                };
                write!(buf,"\t\t{to_fetch:#010x}: {}",display_me).unwrap();
                
            }
        }
        // TODO: Print a little bit of memory context, around where the stack is
        // And some instruction context as well
        environment::write_newline();
        buf

    }
    /// Dump the state of the machine in a simple txt format.
    /// Entries are seperated by newlines, and individual entries all have a : seperator
    pub fn dump_state_txt(&mut self) -> String {
        let mut bytes = String::new();
        write!(bytes,"PC:{:#010x}\n",self.registers.pc.read()).unwrap();
        for i in 0 .. 31 { 
            write!(bytes,"{1:?}:{0:#010x}\n",self.registers.from_num(i).unwrap().read(),self.registers.from_num(i).unwrap().abi_name).unwrap();
        }
        bytes

    }
    /// Return a modifiable list of breakpoints
    pub fn breakpoints(&mut self) -> &mut Vec<u32> {
        &mut self.breakpoints
    }

    
}

#[derive(Error, Debug, Educe)]
// Educe is needed here as some nested error types are not comparable. Comparison is very important
// here for matching and for tests in general, so we skip checking the uncomparable inner fields
// of the relevant error types. This means every IOError is equal to every other IOError, no matter
// the contents of the inner error.
#[educe(PartialEq)]
pub enum ExecutionError {
    #[error("Failed to decode instruction: {0}")]
    ParseError(#[from] std::string::ParseError),
    #[error("Exception while loading value at address {0:#x}")]
    LoadAccessFault(u32),
    #[error("Exception while loading instruction at address {0:#x}")]
    InstructionAccessFault(u32),
    #[error("Tried to read misaligned instruction at {0:#x}")]
    InstructionAddressMisaligned(u32),
    #[error("Breakpoint hit at address {0:#x}")]
    Breakpoint(u32),
    // This isn't really an error, but it is an exceptional condition
    // maybe could be represented a different way but this is easy
    #[error("Successfully finished execution")]
    FinishedExecution(u8),
    #[error("Execution halted by user")]
    HaltedByUser,
    #[error("IO error encountered while running VM: {0}")]
    IOError(#[educe(PartialEq(ignore))] #[from] io::Error),
    #[error("Error while reading line: {0}")]
    ReadlineError(#[educe(PartialEq(ignore))] #[from] ReadlineError),
    #[error("Error while parsing debug command: {0}")]
    DebugParseError(#[from] debugger::DebugParseError),
    #[error("Invalid system call: {0}")]
    InvalidSyscall(u32)
}
/*
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_write_u32() {
        let mut machine = Machine::new(0,Some(0),8,vec![0;4].into_boxed_slice());
        machine.store_word(0xBEE5AA11,0).unwrap();
        for (&mem_value,test_value) in machine.memory.iter().zip([0x11,0xAA,0xE5,0xBE]) {
            assert_eq!(mem_value,test_value);
        }
    }

    #[test]
    fn test_program_completion() {
        let mut machine = Machine::new(0, Some(0), 32, vec![0; 32].into_boxed_slice());
        let store_a0_42 = 0b0010011 | (Register::A0.to_num() << 7) | (42 << 20);
        let _ = machine.store_word(store_a0_42 as u32,0);
        // JALR to RA
        let ret = 0b1100111 | (Register::RA.to_num() << 15) ;
        let _ = machine.store_word(ret as u32,4);

        machine.step().unwrap();
        assert_eq!(machine.step(),Err(ExecutionError::FinishedExecution(42)))


    }
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn load_store_byte_asm(data: u8, s in 16u32..(1<<11)) {
            let mut machine = Machine::new(0, Some(0), s+4, vec![0; s as usize+4].into_boxed_slice());
            let store_a0_42: u32 = 0b0010011 | ((Register::T1.to_num()as u32) << 7) | ((data as u32) << 20);
            let _ = machine.store_word(store_a0_42 as u32,0);
            println!("S: {}",s);

            // Store the data byte in s
            let sb: u32 = 0b0100011 | ((Register::T1.to_num()as u32) << 20) 
                | ((s & 0x1F) << 7) | ((s & 0xFE0)<<20);
            let _ = machine.store_word(sb, 4);
            println!("SB: {:08x}",sb);
            println!("TOP: {:08x}",(s & 0xFE0)<<20);

            let lb: u32 = 0b0000011 | ((Register::A0.to_num()as u32) << 7) | (s << 20);
            let _ = machine.store_word(lb,8);

            // JALR to RA
            let ret = 0b1100111 | (Register::RA.to_num() << 15) ;
            let _ = machine.store_word(ret as u32,12);

            for i in 0 .. 4 {
                println!("{:?}",Operation::from_bytes(machine.read_instruction_bytes(i*4)?));
            }
            machine.step().unwrap();
            machine.step().unwrap();
            machine.step().unwrap();
            assert_eq!(machine.step(),Err(ExecutionError::FinishedExecution(data)))
        }
    }

}
    */
