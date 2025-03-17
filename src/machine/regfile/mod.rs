use clap::Error;
use strum::EnumString;
use std::fmt::Debug;
use crate::Error;


mod register;

pub use register::Register;
pub use register::RegABI;

#[derive (Debug,serde::Serialize)]
pub struct RegisterFile {
    pub pc: Register,
    registers: [Register; 32]
}


impl RegisterFile {
    pub fn new() -> RegisterFile {
        use register::RegABI::*;
        let mut r = RegisterFile {
            pc: Register::new(PC),
            // Order is important!!
            registers: [
                Zero,
                RA,
                SP,
                GP,
                TP,
                T0,
                T1,
                T2,
                S0,
                S1,
                A0,
                A1,
                A2,
                A3,
                A4,
                A5,
                A6,
                A7,
                S2,
                S3,
                S4,
                S5,
                S6,
                S7,
                S8,
                S9,
                S10,
                S11,
                T3,
                T4,
                T5,
                T6
            ].into_iter().map(|abi| {Register::new(abi)}).collect::<Vec<Register>>().try_into().unwrap()
            // this instantiates a new Register "object" (I still have to get used to Rust not having objects) for each of the 32 registers
            // I could have just written Register::new() 32 times but this seems more idiomatic
        };
        r
    }
    
    pub fn from_num(&mut self, reg_num: u32) -> Result<&mut Register,RegisterError> { self.registers.get_mut(reg_num as usize).ok_or(RegisterError::InvalidRegister(reg_num)) }
    pub fn from_abi(&mut self, reg_abi: RegABI) -> &mut Register { self.registers.iter_mut().find(|r| r.abi_name == reg_abi).unwrap() }


    
    

}

#[derive(Error, Debug, PartialEq)]
pub enum RegisterError {
    #[error("Invalid register: {0}")]
    InvalidRegister(u32)
    /*
    #[error("Invalid instruction: {0:#x}")]
    InvalidInstruction(u32),
    #[error("Invalid opcode: {0:#x}")]
    InvalidOpcode(u8),
    #[error("Invalid format: {0:#x}")] // not sure if correct formatting
    InvalidFormat(u32),
    */
}
