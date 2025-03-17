use strum::EnumString;
use std::{error, fmt::Debug};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::convert::TryFrom;
use crate::machine::instruction::ParseError;


#[derive (Debug,serde::Serialize,Default)]
pub struct Register {
    pub abi_name: RegABI,
    pub value: u32,
}


impl Register {
    pub fn new(abi:RegABI) -> Register {
        let mut r = Register {abi_name: abi, value: 0 };
        r
    }
    pub fn write(&mut self, val: u32) {
        // Don't allow writes to the zero register
        if(self.abi_name==RegABI::Zero) { return };
        self.value = val;
    }
    pub fn read(&self) -> u32 {
        return self.value;
    }
}
#[repr(u8)]
#[derive(PartialEq, Debug, EnumString,Clone,Copy,serde::Serialize,Default,TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
pub enum RegABI {
    PC,
    #[default]
    Zero,
    RA,
    SP,
    GP,
    TP,
    T0,
    T1,
    T2,
    #[strum(to_string="S0/FP")]
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
}
impl RegABI {
    pub fn from_num(i:u8) -> Result<RegABI,ParseError> {
        match RegABI::try_from(i+1) {
            Ok(val) => Ok(val),
            Err(e) => Err(ParseError::InvalidRegister(i as u32))
        }
    }
}

