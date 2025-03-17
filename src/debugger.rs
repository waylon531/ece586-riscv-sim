use crate::regfile::Register;
use crate::machine::{Machine,ExecutionError};
use thiserror::Error;
use std::str::FromStr;
use std::fmt;

/// This document describes the commands for the debugger built-in to the simulator.
/// All commands are case insensitive
/// 
/// ```
/// PEEK    [format]    <addr/reg>      # Read data at a memory location or from a register
///                                     # NOTE:  s0 shows the integer in s0
///                                     #       [s0] dereferences s0 and shows memory contents
/// POKE    <data>      <addr/reg>      # Modify data at a memory location or in a register
/// WATCH   [format]    <addr/reg>      # Read data every time control is returned
///                                     # to the debugger
/// STEP    [count]                     # Step once, or the given number of times
/// BREAK   [address]                   # Set a breakpoint at the given address
/// RMBRK   [address/num]               # Remove a breakpoint at the given address
///                                     # or by breakpoint index
/// LSBRK                               # List out all breakpoints
/// CONTINUE                            # Return control to the program and run
///                                     # until a breakpoint is hit
/// RUN                                 # Synonym for CONTINUE
/// EXIT                                # Close the emulator
/// HELP                                # Show this help message

// Maybe should add a string display at some point
// And/or char display
#[derive(Default,Clone,PartialEq,Debug)]
pub enum DisplayFormat {
    #[default]
    Hex,
    Signed,
    Unsigned,
    Binary
}
// This probably should have a way to specify the number of bits
impl DisplayFormat {
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        match s.to_lowercase().as_str() {
            "/x" => Ok(DisplayFormat::Hex),
            "/u" => Ok(DisplayFormat::Unsigned),
            "/i" => Ok(DisplayFormat::Signed),
            "/b" => Ok(DisplayFormat::Binary),
            e => Err(DebugParseError::InvalidFormat(e.to_string()))
        }
    }

}
#[derive(Clone,PartialEq,Debug)]
pub enum Location {
    /// A register number
    Register(Register),
    /// A memory address
    Addr(u32)
}
impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Register(r) => write!(f, "{:?}", r),
            Location::Addr(a) => write!(f, "{a:#x}"),

        }
    }
}

impl Location {
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        match Register::from_str(s) {
            // If in parses as a register the use that
            Ok(reg) => Ok(Location::Register(reg)),
            Err(_) => {
                // Otherwise try hex
                let addr_str = s.trim_start_matches("0x").trim_start_matches("0X");
                Ok(Location::Addr(
                        u32::from_str_radix(addr_str,16)
                        .map_err(|_| DebugParseError::InvalidLocation(s.to_string()))?))
            }

        }
    }
}
#[derive(Clone,PartialEq,Debug)]
pub enum BreakpointIdentifier {
    Addr(u32),
    Index(usize)
}
// These are either an address or an integer index
// Addresses are formatted as 0x.... or 0X....
impl BreakpointIdentifier {
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        // Parse as an address if the string starts with 0x
        if s.to_lowercase().starts_with("0x") {
            Ok(BreakpointIdentifier::Addr(
                    u32::from_str_radix(s.to_lowercase().trim_start_matches("0x"),16)
                         .map_err(|_| DebugParseError::InvalidHex(s.to_string()))?
                         ))


        // Or as a decimal index otherwise
        } else {
            Ok(BreakpointIdentifier::Index(usize::from_str_radix(s,10)
                         .map_err(|_| DebugParseError::InvalidNumber(s.to_string()))?
                         ))
        }
    }

}
#[derive(Clone,PartialEq,Debug)]
pub enum Data {
    Byte(i8),
    Halfword(i16),
    Word(i32)
}
impl Data {
    /// Try parsing into the smallest available type, if hex, a word if decimal, or the given size
    /// with /8 /16 or /32
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        // Whether we should shrink the number down
        let mut fit = false;
        let mut split = s.split("/");
        let s = split.next()
                     .ok_or(DebugParseError::InvalidData(s.to_string()))?
                     .to_lowercase();
        let suffix = split.next();
        let number: u32 = if s.starts_with("0x") {
            // Hex should be shrunk by default
            fit=true;
            u32::from_str_radix(&s.to_lowercase().trim_start_matches("0x"),16)
                         .map_err(|_| DebugParseError::InvalidHex(s.to_string()))?
                         
            
        } else {
            i32::from_str_radix(&s,10)
                         .map_err(|_| DebugParseError::InvalidNumber(s.to_string()))? as u32

        };
        Ok(
        if suffix == Some("8") {
            Data::Byte(number as i8)
        } else if suffix == Some("16") {
            Data::Halfword(number as i16)
        } else if suffix == Some("32") {
            Data::Word(number as i32)
        } else if suffix == None && fit {
            if number <= u8::MAX as u32{
                Data::Byte(number as i8)
            } else if number <= u16::MAX as u32{
                Data::Halfword(number as i16)
            } else {
                Data::Word(number as i32)
            }

        } else if let Some(e) = suffix {
            return Err(DebugParseError::FailedToParseSuffix(e.to_string()))
        } else  {
            Data::Word(number as i32)

        }
        )
    }
}
#[derive(Error,Debug, PartialEq, Eq)]
pub enum DebugParseError {
    #[error("Tried to parse empty string")]
    Empty,
    #[error("Not enough arguments")]
    NotEnoughArguments,
    #[error("Too many arguments")]
    TooManyArguments,
    #[error("Invalid command `{0}`")]
    InvalidCommand(String),
    #[error("Invalid data `{0}`")]
    InvalidData(String),
    #[error("Invalid format specifier `{0}`")]
    InvalidFormat(String),
    #[error("Invalid address or location `{0}`")]
    InvalidLocation(String),
    #[error("Failed to parse number, found `{0}`")]
    InvalidNumber(String),
    #[error("Failed to parse suffix `{0}`")]
    FailedToParseSuffix(String),
    #[error("Invalid hex number `{0}`")]
    InvalidHex(String),
}
#[derive(Clone,PartialEq,Debug)]
pub enum DebugCommand {
    PEEK(DisplayFormat,Location),
    WATCH(DisplayFormat,Location),
    RMWATCH(Location),
    // Depending on number of characters in data will pick byte, halfword, or word, smallest first
    POKE(Data,Location),
    STEP(usize),
    BREAK(u32),
    RMBRK(BreakpointIdentifier),
    LSBRK,
    CONTINUE,
    HELP,
    EXIT


}

impl DebugCommand {
    pub fn from_string(s: &str) -> Result<Self,DebugParseError> {
        let mut iterator = s.trim().split(' ');
        let first = iterator.next().ok_or(DebugParseError::Empty)?.to_lowercase();
        let mut rest: Vec<&str> = iterator.collect();
        let command = match first.as_str() {
            "peek" => {
                if rest.len() > 2 { return Err(DebugParseError::TooManyArguments) };
                let location = Location::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                let format = match rest.pop() {
                    Some(s) => DisplayFormat::parse(s)?,
                    None => DisplayFormat::default()
                };
                DebugCommand::PEEK(format,location)
            },
            "watch" => {
                if rest.len() > 2 { return Err(DebugParseError::TooManyArguments) };
                let location = Location::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                let format = match rest.pop() {
                    Some(s) => DisplayFormat::parse(s)?,
                    None => DisplayFormat::default()
                };
                DebugCommand::WATCH(format,location)
            },
            "rmwatch" => {
                if rest.len() > 1 { return Err(DebugParseError::TooManyArguments) };
                let location = Location::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                DebugCommand::RMWATCH(location)
            },
            "poke" => {
                if rest.len() > 2 { return Err(DebugParseError::TooManyArguments) };
                let data = Data::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                let location = Location::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                DebugCommand::POKE(data,location)

            },
            "step" => {
                if rest.len() > 1 { return Err(DebugParseError::TooManyArguments) };
                match rest.pop() {
                    None => DebugCommand::STEP(1),
                    Some(num_str) => {
                        let num = usize::from_str(num_str)
                                        .map_err(|_| DebugParseError::InvalidNumber(num_str.to_owned()))?;
                        DebugCommand::STEP(num)
                    }

                }
            },
            "break" => {
                if rest.len() > 2 { return Err(DebugParseError::TooManyArguments) };
                let location = rest.pop().ok_or(DebugParseError::NotEnoughArguments)?;
                let address = location.trim_start_matches("0x").trim_start_matches("0X");
                DebugCommand::BREAK(
                        u32::from_str_radix(address,16)
                        .map_err(|_| DebugParseError::InvalidHex(address.to_string()))?)

            },
            "rmbrk" => {
                if rest.len() > 1 { return Err(DebugParseError::TooManyArguments) };
                let brk = BreakpointIdentifier::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                DebugCommand::RMBRK(brk)

            },
            "help" => DebugCommand::HELP,
            "exit" => DebugCommand::EXIT,
            "lsbrk" => DebugCommand::LSBRK,
            "run" | "continue" => DebugCommand::CONTINUE,
            c@ _ => return Err(DebugParseError::InvalidCommand(c.to_string()))

        };
        Ok(command)
    }
    /// Return usage information for the debugger as a list of strings
    pub fn usage() -> &'static [&'static str] {
&[
"PEEK    [format]    <addr/reg>      # Read data at a memory location or from a register",
"                                    # Valid formats are /x (hex), /u (unsigned),",
"                                    # /i (integer), and /b (binary)",
"POKE    <addr/reg>      <data>      # Modify data at a memory location or in a register",
"WATCH   [format]    <addr/reg>      # Read data every time control is returned",
"                                    # to the debugger",
"RMWATCH             <addr/reg>      # Stop watching a variable",
"STEP    [count]                     # Step once, or the given number of times",
"BREAK   [address]                   # Set a breakpoint at the given address",
"RMBRK   [address/num]               # Remove a breakpoint at the given address",
"                                    # or by breakpoint index",
"LSBRK                               # List out all breakpoints",
"CONTINUE                            # Return control to the program and run",
"                                    # until a breakpoint is hit",
"RUN                                 # Synonym for CONTINUE",
"EXIT                                # Close the emulator",
"HELP                                # Show this help message",
]


    }
    
    pub fn execute(
        &self, 
        machine: &mut Machine, 
        should_step: &mut Option<usize>,
        should_trigger_cmd: &mut bool,
        run: &mut bool,
        watchlist: &mut Vec<DebugCommand>
    ) -> Result<Vec<String>, ExecutionError> {
        let mut status = Vec::new();
        match self {
            &DebugCommand::EXIT => return Err(ExecutionError::HaltedByUser),
            &DebugCommand::HELP => for line in DebugCommand::usage() { status.push(line.to_string()) },
            &DebugCommand::CONTINUE => {
                *should_trigger_cmd = false; 
                *run = true;
            },
            &DebugCommand::STEP(count) => {
                *should_trigger_cmd = false;
                *should_step = Some(count);
                *run = true;
            },
            &DebugCommand::BREAK(addr) => {
                if !machine.breakpoints().contains(&addr) {
                    machine.breakpoints().push(addr)
                } else {
                    status.push(
                        format!("Unable to insert duplicate breakpoint at {0:#010x}", addr));
                }
                status.push(format!("Added breakpoint {} at {:#010x}",machine.breakpoints().len(),addr));

            },
            &DebugCommand::RMBRK(BreakpointIdentifier::Index(index)) => {
                machine.breakpoints().remove(index);
                status.push(format!("Successfully removed breakpoint {}",index));
            },
            &DebugCommand::RMBRK(BreakpointIdentifier::Addr(address)) => {
                let mut to_remove = None;
                for (k,v) in machine.breakpoints().iter().enumerate() {
                    if *v == address {
                        to_remove = Some(k);
                        break;
                    }
                }
                if let Some(key) = to_remove {
                    machine.breakpoints().remove(key);
                    status.push(format!("Successfully removed breakpoint {}",key));
                } else {
                    status.push("Unable to find breakpoint".to_string());
                };
            },
            &DebugCommand::LSBRK => {
                for (index,address) in machine.breakpoints().iter().enumerate() {
                    status.push(format!("{index}: {address:#010x}"));
                }
            },

            &DebugCommand::POKE(Data::Byte(data),Location::Addr(addr)) => {
                match machine.store_byte(data as u8,addr) {
                    Ok(()) => (),
                    Err(e) => {
                        status.push(format!("Failed to store data with error {}",e));
                    }
                }
            },
            &DebugCommand::POKE(Data::Halfword(data),Location::Addr(addr)) => {
                match machine.store_halfword(data as u16,addr) {
                    Ok(()) => (),
                    Err(e) => {
                        status.push(format!("Failed to store data with error {}",e));
                    }
                }
            },
            &DebugCommand::POKE(Data::Word(data),Location::Addr(addr)) => {
                match machine.store_word(data as u32,addr) {
                    Ok(()) => (),
                    Err(e) => {
                        status.push(format!("Failed to store data with error {}",e));
                    }
                }
            },
            &DebugCommand::POKE(ref data,Location::Register(reg)) => {
                let converted_data = match data {
                    &Data::Byte(d) => d as u32,
                    &Data::Halfword(d) => d as u32,
                    &Data::Word(d) => d as u32,
                };
                machine.set_reg(reg,converted_data);
            },
            DebugCommand::PEEK(fmt,location) => {
                let data = match location {
                    Location::Register(reg) => machine.get_reg(*reg),
                    Location::Addr(a) => match machine.read_word(*a) {
                        Ok(data) => data,
                        Err(e) => {
                            status.push(format!("Failed to read data with error {}",e));
                            return Ok(status);
                        }
                    }
                };
                status.push(match fmt {
                    DisplayFormat::Hex => format!("{location}: {:#X}",data),
                    DisplayFormat::Unsigned => format!("{location}: {}",data as u32),
                    DisplayFormat::Signed => format!("{location}: {}",data as i32),
                    DisplayFormat::Binary => format!("{location}: {:#b}",data),

                });
            },
            // NOTE: This is kind of a hacky way to do the watchlist but it saves on code
            // currently it consists of a buch of debugcommands that will be executed each
            // time the status is printed, but generally only commands that will just print
            // something. Hopefully CONTINUEs never make it in here.
            DebugCommand::WATCH(fmt,location) => {
                watchlist.push(DebugCommand::PEEK(fmt.clone(),location.clone()));
            },
            DebugCommand::RMWATCH(location) => {
                *watchlist = watchlist.clone()
                                      .into_iter()
                                      .filter(|c| match c {
                                          DebugCommand::PEEK(_fmt,loc) => loc != location,
                                          _ => false

                                      })
                                      .collect();
            },
        }
        return Ok(status);
    }
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_too_many_args() {
        assert_eq!(DebugCommand::from_string("step 1 2"), Err(DebugParseError::TooManyArguments));
    }

    #[test]
    fn test_parse_step() {
        assert_eq!(DebugCommand::from_string("step 125"), Ok(DebugCommand::STEP(125)));
    }

    #[test]
    fn test_parse_peek() {
        assert_eq!(DebugCommand::from_string("peek /x 0xDEAD"), 
                   Ok(DebugCommand::PEEK(DisplayFormat::Hex,Location::Addr(0xDEAD))));
    }
    #[test]
    fn test_parse_poke() {
        assert_eq!(DebugCommand::from_string("poke 0xDEAD -123"), 
                   Ok(DebugCommand::POKE(Data::Word(-123),Location::Addr(0xDEAD))));
    }
    #[test]
    fn test_parse_poke_reg() {
        assert_eq!(DebugCommand::from_string("poke t2 -123"), 
                   Ok(DebugCommand::POKE(Data::Word(-123),Location::Register(Register::T2))));
    }

    #[test]
    fn test_parse_poke_size() {
        assert_eq!(DebugCommand::from_string("poke t2 0xFF"), 
                   Ok(DebugCommand::POKE(Data::Byte(0xFFu8 as i8),Location::Register(Register::T2))));
        assert_eq!(DebugCommand::from_string("poke t2 0xFFFF"), 
                   Ok(DebugCommand::POKE(Data::Halfword(0xFFFFu16 as i16),Location::Register(Register::T2))));
        assert_eq!(DebugCommand::from_string("poke t2 0xFFFFFFFF"), 
                   Ok(DebugCommand::POKE(Data::Word(0xFFFFFFFFu32 as i32),Location::Register(Register::T2))));
        assert_eq!(DebugCommand::from_string("poke t2 0xFF/8"), 
                   Ok(DebugCommand::POKE(Data::Byte(0xFFu8 as i8),Location::Register(Register::T2))));
        assert_eq!(DebugCommand::from_string("poke t2 0xFF/16"), 
                   Ok(DebugCommand::POKE(Data::Halfword(0xFFu16 as i16),Location::Register(Register::T2))));
        assert_eq!(DebugCommand::from_string("poke t2 0xFF/32"), 
                   Ok(DebugCommand::POKE(Data::Word(0xFFu32 as i32),Location::Register(Register::T2))));
    }
}
