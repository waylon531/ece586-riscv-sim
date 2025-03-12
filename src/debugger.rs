use crate::register::Register;
use thiserror::Error;
use std::str::FromStr;

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
#[derive(Default,Clone)]
pub enum DisplayFormat {
    #[default]
    Hex,
    Decimal,
    Binary
}
impl DisplayFormat {
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        match s.to_lowercase().as_str() {
            "/x" => Ok(DisplayFormat::Hex),
            "/d" => Ok(DisplayFormat::Decimal),
            "/b" => Ok(DisplayFormat::Binary),
            e => Err(DebugParseError::InvalidFormat(e.to_string()))
        }
    }

}
#[derive(Clone)]
pub enum Location {
    /// A register number
    Register(Register),
    /// A memory address
    Addr(u32)
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
#[derive(Clone)]
pub enum BreakpointIdentifier {
    Addr(u32),
    Index(usize)
}
impl BreakpointIdentifier {
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {

        unimplemented!()
    }

}
#[derive(Clone)]
pub enum Data {
    Byte(i8),
    Halfword(i16),
    Word(i32)
}
impl Data {
    /// Try parsing into the smallest available type, if hex, a word if decimal, or the given size
    /// with /8 /16 or /32
    pub fn parse(s: &str) -> Result<Self,DebugParseError> {
        if let Some(byte) = s.strip_suffix("/8") {

        }
        unimplemented!()
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
    #[error("Invalid format specifier `{0}`")]
    InvalidFormat(String),
    #[error("Invalid address or location `{0}`")]
    InvalidLocation(String),
    #[error("Failed to parse number, found `{0}`")]
    InvalidNumber(String),
    #[error("Invalid hex number `{0}`")]
    InvalidHex(String),
}
#[derive(Clone)]
pub enum DebugCommand {
    PEEK(DisplayFormat,Location),
    WATCH(DisplayFormat,Location),
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
            "poke" => {
                if rest.len() > 2 { return Err(DebugParseError::TooManyArguments) };
                let location = Location::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
                let data = Data::parse(rest.pop().ok_or(DebugParseError::NotEnoughArguments)?)?;
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
"                                    # NOTE:  s0 shows the integer in s0",
"                                    #       [s0] dereferences s0 and shows memory contents",
"POKE    <data>      <addr/reg>      # Modify data at a memory location or in a register",
"WATCH   [format]    <addr/reg>      # Read data every time control is returned",
"                                    # to the debugger",
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
}


