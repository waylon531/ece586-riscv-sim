
use std::ops::Sub;

use crate::Error;
use crate::machine::RegABI;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Invalid register: {0}")]
    InvalidRegister(u32),
    #[error("Invalid instruction: {0:#x}")]
    InvalidInstruction(u32),
    #[error("Invalid opcode: {0:#x}")]
    InvalidOpcode(u8),
    #[error("Invalid format: {0:#x}")] // not sure if correct formatting
    InvalidFormat(u32),
}

// sign-extend imm field and convert to i32
pub fn sign_extend(num: u32, bits: u8) -> u32 {
    let shamt: u8 = 32 - bits;
    let res = ((num as u32) << shamt) >> shamt;
    res
}