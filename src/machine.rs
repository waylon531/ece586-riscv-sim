use crate::debugger::{DebugCommand,self};
use crate::decode::ParseError;
use crate::devices::{Device};
use crate::opcode::Operation;
use crate::register::Register;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rustyline::error::ReadlineError;
use serde::Serialize;
use std::fmt::Write;
use std::io::{self,Stdin};
use single_value_channel::Updater as SvcSender;
use crossbeam_channel::Receiver as CbReceiver;
use educe::Educe;

use thiserror::Error;

use crate::statetransfer::{self, ControlCode, MachineState};
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
    memory: Box<[u8]>,
    // The top of memory, points right above the last usable address
    memory_top: u32,
    // Store x1-x31
    // x0 is always 0, no reason to store
    registers: [u32; 31],
    pc: u32,
    // Whether we should step over a breakpoint or not
    pass_breakpoint: bool,
    // NOTE: A vec means we linear search through breakpoints, but also gives a map from 
    //       index/breakpoint number to breakpoint which is nice for ui
    //
    //       Might be way slow though to iterate through this every cycle though
    breakpoints: Vec<u32>,
    // store the current instruction as a string, for display
    cur_inst: String,
    // store the memory locations that changed since last instruction
    memory_changes: Vec<(u32,u32)>,
    // is the machine running?
    web_runfullspeed: bool,
    // should we run the next step? (yes I'm VERY aware these conflict with existing variables for the debugger - call it a merge conflict; TODO: consolidate)
    web_step: bool,
    verbose: bool,
    cycle: u128,
    #[serde(skip_serializing)]
    devices: Vec<Device>,
    #[serde(skip_serializing)]
    env: Environment,
}
impl Machine {
    pub fn new(starting_addr: u32, stack_addr: Option<u32>, memory_top: u32, memmap: Box<[u8]>,verbose:bool,devices:Vec<Device>) -> Self{
        let mut m = Machine {
                    memory: memmap,
                    registers: [0;31],
                    memory_top,
                    devices,
                    pc: starting_addr,
                    pass_breakpoint: false,
                    breakpoints: Vec::new(),
                    env:Environment::new(),
                    cur_inst: "".to_string(),
                    memory_changes: Vec::new(),
                    web_runfullspeed: true,
                    web_step: false,
                    verbose: verbose,
                    cycle: 0
                    
        };
        // Set the stack pointer to the lowest invalid memory address by default, aligning down to
        // nearest 16 bytes
        m.set_reg(Register::SP, stack_addr.unwrap_or(memory_top & !(0xF)));
        m
    }
    /// Run the machine til completion, either running silently until an error is hit or bringing
    /// up the debugger after every step
    pub fn run(&mut self, single_step: bool, _stdin: &Stdin, commands_rx: Option<CbReceiver<statetransfer::ControlCode>>, state_tx: Option<SvcSender<statetransfer::MachineState>>) -> Result<(),ExecutionError> {
        // reset timer
        self.env.reset_timer();
        // NOTE: this cannot be a global include as it conflicts with fmt::Write;
        
        let should_trigger_cmd = Arc::new(AtomicBool::new(single_step));
        let mut rl = rustyline::DefaultEditor::new()?;
        // Set the default command to step, by default
        let mut last_cmd = DebugCommand::STEP(1);
        let mut should_step = None;
        // Status messages to print
        let mut status: Vec<String> = Vec::new();
        let mut watchlist: Vec<DebugCommand> = Vec::new();
        if ! commands_rx.is_none() {
            self.web_runfullspeed = false;
        };
        loop {
            // Stringify the current instruction
            self.cur_inst = match self.read_instruction_bytes(self.pc) {
                Ok(bytes) => match Operation::from_bytes(bytes) {
                    Ok(op) => format!("{}",op),
                    _ => "invalid".to_owned()
                },
                _ => "invalid".to_owned()
            };
            /*
                At the start of each cycle, if the web server is running, we want to communicate with it.
                We exchange information - we send the current state of the machine, and we read commands sent from the web interface.
                Those commands may include modifications to registers or memory addresses - so we interpret those.
             */
            match state_tx {
                Some(ref tx) => {
                    // send machine state to web ui
                    let m = MachineState {
                        pc: self.pc,
                        registers: self.registers.clone(),
                        cur_inst: self.cur_inst.clone(),
                        memory_changes: self.memory_changes.clone(),
                        cycle: self.cycle
                    };
                    tx.update(m).unwrap();
                },
                None => {}
            }
            let mut web_commands: Vec<ControlCode> = Vec::new();
            match commands_rx {
                Some(ref rx) => {
                    rx.try_iter().for_each(|c| web_commands.push(c));
                },
                None => {}
            }
            web_commands.iter().for_each(|c| 
                match c {
                    ControlCode::STOP => { self.web_runfullspeed = false; }
                    ControlCode::RUN => { self.web_runfullspeed = true; }
                    ControlCode::STEP => { self.web_step = true }
                    ControlCode::POKEREG { register, value } => {
                        self.registers[register.to_num()] = *value;
                    },
                    ControlCode::POKE { address, value } => {
                        self.memory[*address as usize] = *value as u8;
                    }
                    ControlCode::JMP { address } => {
                        self.pc = *address;
                    },
                    _ => {}
                }
            );
            // Check if we stepping for N times, and if we are at the end then pull the debugger
            // back up
            // Otherwise decrement the step counter
            if let Some(count) = should_step {
                if count <= 1 {
                    should_trigger_cmd.store(true,Ordering::SeqCst);
                    should_step = None;
                } else {
                    should_step = Some(count-1);
                }
            };

            if should_trigger_cmd.load(Ordering::SeqCst) {
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
                        &mut false,
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
                
                let mut value = should_trigger_cmd.load(Ordering::SeqCst);
                let mut new_status = command.execute(
                    self,
                    &mut should_step, 
                    &mut value,
                    &mut run,
                    &mut watchlist
                    )?;
                should_trigger_cmd.store(value,Ordering::SeqCst);


                // Combine vecs
                for line in new_status.drain(..) {
                    status.push(line);
                }

                // Many commands will instantly return control back to the prompt without
                // stepping execution at all
                if !run { // horrible
                    continue;
                }

                // Note: this will only get updated on commands that step or continue
                //       not 100% sure that's what we want yet
                last_cmd = command;

            }
            if !(self.web_runfullspeed || self.web_step) { continue };
            match self.step() {
                Ok(()) => {},
                // Should errors bail? Or bring up the debugger to explore program state?
                // Bail for now probably, its easier (though worse)
                Err(e@ ExecutionError::Breakpoint(_)) => {
                    should_trigger_cmd.store(true,Ordering::SeqCst);
                    // Give a pass so the next step of execution can make it past the breakpoint
                    self.pass_breakpoint = true;
                    status.push(format!("{}",e));
                },
                Err(e) => return Err(e)

            }
            if self.verbose {
                println!("              cycle {}", self.cycle);
                print!("{}", self.dump_state_txt());
            }
            self.web_step = false;
            self.cycle+=1;
            // Check if the user is pressing ctrl-c, and if they are, drop back into the debugger
            // oops this needs a bonus thread, this is going to suck
            // the thread can get spun up whenever we are running and spun down, or paused, when we
            // go to the debugger
           
        }
        
    }
    // String formatting should never fail, it's likely safe to unwrap here
    pub fn display_info(&self) -> String {
        let mut buf = String::new();
        // Why am I doing this crazy nonsense? To ensure we only print terminal control characters if the output is a terminal.
        if environment::which_new_line() == "\r\n" { write!(buf,"{}","\r").unwrap(); };
        write!(buf,"PC:\t  {:#010x}", self.pc).unwrap();
        for i in 0 .. 31 {
            write!(buf,"{}",environment::which_new_line()).unwrap();
            write!(buf,"{1:?}:\t{0:>12}\t{0:#010x}",self.registers[i],Register::from_num((i as u32)+1).unwrap()).unwrap();
            if i < 16 {
                let context: i32 = (i as i32-8)*4;
                let to_fetch = self.pc.overflowing_add_signed(context);
                match to_fetch {
                    (addr, false) => {
                        let display_me = match self.read_instruction_bytes(addr) {
                            Ok(bytes) => match Operation::from_bytes(bytes) {
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
                let to_fetch = self.registers[Register::SP].saturating_add(context as u32);
                let display_me = match self.read_word(to_fetch) {
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
    pub fn dump_state_txt(&self) -> String {
        let mut bytes = String::new();
        write!(bytes,"PC:{:#010x}\n",self.pc).unwrap();
        for i in 0 .. 31 { 
            write!(bytes,"{1:?}:{0:#010x}\n",self.registers[i],Register::from_num((i as u32)+1).unwrap()).unwrap();
        }
        bytes

    }
    /// Return a modifiable list of breakpoints
    pub fn breakpoints(&mut self) -> &mut Vec<u32> {
        &mut self.breakpoints
    }
    pub fn get_reg(&mut self,reg: Register) -> u32 {
        self.registers[reg]
    }
    pub fn set_reg(&mut self,reg: Register, value: u32) {
        let reg_num = reg.to_num();
        // Writes to the zero register are NOPs
        if reg_num == 0 {
            return;
        } else {
            self.registers[reg_num - 1] = value;
        }
    }

    // These 4 functions could probably be more modular ...
    pub fn read_instruction_bytes(&self, addr: u32) -> Result<&[u8], ExecutionError> {
        // Error out if the address is not aligned on a 32-bit boundary
        if addr & 0b11 != 0 {
            Err(ExecutionError::InstructionAddressMisaligned(addr))
        // If the memory top is zero then assume we are using the full 4GB address space as memory
        } else if self.memory_top == 0 || addr.overflowing_add(4).0 <= self.memory_top {
            Ok(&self.memory[addr as usize .. addr as usize + 4])
        } else {
            Err(ExecutionError::InstructionAccessFault(addr))
        }
    }
    pub fn read_byte(&self, addr: u32) -> Result<i8, ExecutionError> {
        // Check to see if the addr points to a memory mapped device
        // The device space starts with 0xF
        if addr >> 28 == 0xF {
            let device_addr = addr & 0xFFFFFFF;
            for device in self.devices.iter() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.read_byte(device_addr).map_err(|e| ExecutionError::DeviceError(e))?)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr < self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as i8)
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_word(&self, addr: u32) -> Result<u32, ExecutionError> {
        if addr >> 28 == 0xF {
            let device_addr = addr & 0xFFFFFFF;
            for device in self.devices.iter() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.read_word(device_addr).map_err(|e| ExecutionError::DeviceError(e))? as u32)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as u32
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u32) << 8)
                + ((self.memory[addr.overflowing_add(2).0 as usize] as u32) << 16)
                + ((self.memory[addr.overflowing_add(3).0 as usize] as u32) << 24))
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_halfword(&self, addr: u32) -> Result<i16, ExecutionError> {
        if addr >> 28 == 0xF {
            let device_addr = addr & 0xFFFFFFF;
            for device in self.devices.iter() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.read_halfword(device_addr).map_err(|e| ExecutionError::DeviceError(e))?)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            Ok((self.memory[addr as usize] as u16
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u16) << 8)) as i16)
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_byte(&mut self, data: u8, addr: u32) -> Result<(), ExecutionError> {
        if addr >> 28 == 0xF {
            let device_addr = addr & 0xFFFFFFF;
            for device in self.devices.iter_mut() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.store_byte(device_addr,data).map_err(|e| ExecutionError::DeviceError(e))?)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr < self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_halfword(&mut self, data: u16, addr: u32) -> Result<(), ExecutionError> {
        if addr >> 28 == 0xF {
            let device_addr = addr & 0xFFFFFFF;
            for device in self.devices.iter_mut() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.store_halfword(device_addr,data).map_err(|e| ExecutionError::DeviceError(e))?)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }

    pub fn store_word(&mut self, data: u32, addr: u32) -> Result<(), ExecutionError> {
        let device_addr = addr & 0xFFFFFFF;
        if addr >> 28 == 0xF {
            for device in self.devices.iter_mut() {
                if device.memory_range().contains(&device_addr) {
                    return Ok(device.store_word(device_addr,data).map_err(|e| ExecutionError::DeviceError(e))?)
                }
            }
            Err(ExecutionError::LoadAccessFault(addr))
        } else if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            self.memory[addr.overflowing_add(2).0 as usize] = (data >> 16) as u8;
            self.memory[addr.overflowing_add(3).0 as usize] = (data >> 24) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    // Fetch, decode, and execute an instruction
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        use Operation::*;
        // First, check if we're at a breakpoint, and cannot pass over it
        if self.breakpoints.contains(&self.pc) && !self.pass_breakpoint {
            return Err(ExecutionError::Breakpoint(self.pc));
        } else {
            // Unset breakpoint pass
            self.pass_breakpoint = false;

        }
        //Fetch and decode
        let op = Operation::from_bytes(self.read_instruction_bytes(self.pc)?)?;

        // Branches and jumps will set this to false
        let mut increment_pc = true;

        match op {
            ADDI(rd, rs1, imm) => {
                self.set_reg(rd, self.registers[rs1].overflowing_add(imm as u32).0)
            }
            SLTI(rd, rs1, imm) => self.set_reg(
                rd,
                if (self.registers[rs1] as i32) < imm {
                    1
                } else {
                    0
                },
            ),
            SLTIU(rd, rs1, imm) => self.set_reg(
                rd,
                if self.registers[rs1] < (imm as u32) {
                    1
                } else {
                    0
                },
            ),
            ANDI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] & (imm as u32)),
            ORI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] | (imm as u32)),
            XORI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] ^ (imm as u32)),
            SLLI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] << (imm as u32)),
            SRLI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] >> (imm as u32)),
            // Arithmetic shifts are the default on signed numbers, so convert types a few times
            SRAI(rd, rs1, imm) => {
                self.set_reg(rd, ((self.registers[rs1] as i32) >> (imm as u32)) as u32)
            }

            // NOTE: It is assumed that during decoding, imm is shifted 12 bits over and as such
            // is already in the right format here
            LUI(rd, imm) => self.set_reg(rd, imm as u32),
            AUIPC(rd, imm) => self.set_reg(rd, self.pc.overflowing_add(imm as u32).0),

            // Integer, register, register instructions
            // RD first, then SRC1, then SRC2
            ADD(rd, rs1, rs2) => self.set_reg(
                rd,
                self.registers[rs1].overflowing_add(self.registers[rs2]).0,
            ),
            SLTU(rd, rs1, rs2) => self.set_reg(
                rd,
                if self.registers[rs1] < self.registers[rs2] {
                    1
                } else {
                    0
                },
            ),
            SLT(rd, rs1, rs2) => self.set_reg(
                rd,
                if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {
                    1
                } else {
                    0
                },
            ),
            AND(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] & self.registers[rs2]),
            OR(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] | self.registers[rs2]),
            XOR(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] ^ self.registers[rs2]),
            SLL(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] << self.registers[rs2]),
            SRL(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] >> self.registers[rs2]),
            SUB(rd, rs1, rs2) => self.set_reg(
                rd,
                self.registers[rs1].overflowing_sub(self.registers[rs2]).0,
            ),
            SRA(rd, rs1, rs2) => self.set_reg(
                rd,
                ((self.registers[rs1] as i32) >> self.registers[rs2]) as u32,
            ),

            // Control transfer instructions
            // Normal, unconditional jumps use x0 as the register
            JAL(rd, imm) => {
                self.set_reg(rd, self.pc.overflowing_add(4).0);
                // Set the pc, clearing the last bit
                self.pc = self.pc.overflowing_add(imm as u32).0 & (! 0x1); 
                increment_pc = false;
            }
            JALR(rd, rs1, imm) => {
                // A RET to address zero will stop execution
                // The a0 register contains the status code to return
                if rs1 == Register::RA && self.registers[Register::RA] == 0 {
                    return Err(ExecutionError::FinishedExecution(self.registers[Register::A0] as u8))
                }
                self.set_reg(rd,self.pc.overflowing_add(4).0);
                // Add imm to rs1 and zero out lowest bit
                self.pc = self.registers[rs1].overflowing_add(imm as u32).0 & (!1);
                increment_pc = false;
            }

            // Conditional branches
            BEQ(rs1, rs2, imm) => {
                if self.registers[rs1] == self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BNE(rs1, rs2, imm) => {
                if self.registers[rs1] != self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BLT(rs1, rs2, imm) => {
                if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BLTU(rs1, rs2, imm) => {
                if self.registers[rs1] < self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BGE(rs1, rs2, imm) => {
                if (self.registers[rs1] as i32) >= (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BGEU(rs1, rs2, imm) => {
                if self.registers[rs1] >= self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }

            // Loads and stores
            LW(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_word(self.registers[rs1].overflowing_add_signed(imm).0)?,
            ),
            // There are two casts here, one to sign extend and one to put it into the correct type
            LH(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_halfword(self.registers[rs1].overflowing_add_signed(imm).0)? as i32
                    as u32,
            ),
            LHU(rd, rs1, imm) => self.set_reg(
                rd,
                // Note the two as statements here, with just as 32 sign extension will still
                // happen
                self.read_halfword(self.registers[rs1].overflowing_add_signed(imm).0)? as u16 as u32,
            ),
            LB(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_byte(self.registers[rs1].overflowing_add_signed(imm).0)? as i32 as u32,
            ),
            LBU(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_byte(self.registers[rs1].overflowing_add_signed(imm).0)? as u8 as u32,
            ),

            SW(rs1, rs2, imm) => self.store_word(
                self.registers[rs2],
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,
            SH(rs1, rs2, imm) => self.store_halfword(
                (self.registers[rs2] & 0xFFFF) as u16,
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,
            SB(rs1, rs2, imm) => self.store_byte(
                (self.registers[rs2] & 0xFF) as u8,
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,

            // Division


            // Evironment call/syscall
            ECALL => {
                /* Fun with system calls! I think this is technically a BIOS? */
                // this should definitely be its own module I feel
                // a7: syscall, a0-a2: arguments
                match self.env.syscall(self.registers[Register::A7], self.registers[Register::A0],self.registers[Register::A1],self.registers[Register::A2], &mut self.memory) {
                    Ok(result) => {self.set_reg(Register::A0, result as u32)},
                    Err(e) => { return Err(e) }
                }
                
            }

            // Breakpoint for us
            EBREAK => return Err(ExecutionError::Breakpoint(self.pc)),

            // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
            NOP => {}

            // Generic performance hint, we don't need to store any information for them
            // and they are effectively NOPs
            // Same with FENCE
            HINT | FENCE => {}

            /* Multiplication / Division */ 
            MUL(rd, rs1, rs2) => {
                self.set_reg(
                    rd,
                    ((self.registers[rs1] as i32) * (self.registers[rs2] as i32)) as u32,
                )
            },
            MULH(rd, rs1, rs2) => {
                let res: i64 = (self.registers[rs1] as i64) * (self.registers[rs2] as i64) as i64;
                self.set_reg(
                    rd,
                    (res >> 32) as u32,
                )
            },
            MULSU(rd, rs1, rs2) => {
                let res: i64 = (self.registers[rs1] as i64) * (self.registers[rs2] as u64) as i64;
                self.set_reg(
                    rd,
                    (res >> 32) as u32,
                )
            },
            MULU(rd, rs1, rs2) => {
                let res: u64 = (self.registers[rs1] as u64) * (self.registers[rs2] as u64) as u64;
                self.set_reg(
                    rd,
                    (res >> 32) as u32,
                )
            },
            DIV(rd, rs1, rs2) => {
                let res:i32;
                if self.registers[rs2] == 0 {
                    res = (0 - 1) as i32;
                } else {
                    res = (self.registers[rs1] as i32) / (self.registers[rs2] as i32);
                }
                self.set_reg(
                    rd,
                    res as u32
                )
            },
            DIVU(rd, rs1, rs2) => {
                let res;
                if self.registers[rs2] == 0 {
                    res = (0 - 1) as u32;
                } else {
                    res = (self.registers[rs1] as u32) / (self.registers[rs2] as u32);
                }
                self.set_reg(
                    rd,
                    res
                )
            },
            REM(rd, rs1, rs2) => {
                let res: i32;
                if self.registers[rs2] == 0 {
                    res = self.registers[rs1].clone() as i32;
                } else {
                    res = (self.registers[rs1] as i32)  % (self.registers[rs2] as i32);
                }
                self.set_reg(
                    rd,
                    res as u32
                )
            },
            REMU(rd, rs1, rs2) => {
                let res;
                if self.registers[rs2] == 0 {
                    res = (0 - 1) as u32;
                } else {
                    res = (self.registers[rs1] as u32) % (self.registers[rs2] as u32);
                }
                self.set_reg(
                    rd,
                    res
                )
            },


        }

        if increment_pc {
            self.pc = self.pc.overflowing_add(4).0;
        }

        
        Ok(())
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
    ParseError(#[from] ParseError),
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
    #[error("Problem with device: {0}")]
    DeviceError(#[educe(PartialEq(ignore))] Box<dyn std::error::Error>),
    #[error("Invalid system call: {0}")]
    InvalidSyscall(u32)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_write_u32() {
        let mut machine = Machine::new(0,Some(0),8,vec![0;4].into_boxed_slice(),false,Vec::new());
        machine.store_word(0xBEE5AA11,0).unwrap();
        for (&mem_value,test_value) in machine.memory.iter().zip([0x11,0xAA,0xE5,0xBE]) {
            assert_eq!(mem_value,test_value);
        }
    }

    #[test]
    fn test_program_completion() {
        let mut machine = Machine::new(0, Some(0), 32, vec![0; 32].into_boxed_slice(),false, Vec::new());
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
            let mut machine = Machine::new(0, Some(0), s+4, vec![0; s as usize+4].into_boxed_slice(),false, Vec::new());
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
