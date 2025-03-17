use std::ops::Sub;

use crate::Error;
use crate::machine::RegABI;
use super::{extended, SubFields,InstType};

use super::Inst;


pub fn get_subfields (iword: u32, itype: InstType) -> Result<SubFields,ParseError> {
    let mut imm: Option<u32> = None;
    // I don't like these magic numbers either
    // These fields are always in the same position (and they don't matter if they're not used). Important thing is registers and immediate value.
    let mut rs1: Option<RegABI> = None;
    let mut rs2: Option<RegABI> = None;
    let mut rd: Option<RegABI> = None;
    let mut exten: Option<extended::ExtSubfields> = None;
    let mut funct3: Option<u8> = None;
    let mut funct7: Option<u8> = None;
    let rs1i = bitrange(iword, 15, 19) as u8;
    let rs2i = bitrange(iword, 20, 24) as u8;
    let rdi = bitrange(iword, 7, 11) as u8;
    let funct3i: u8 = bitrange(iword, 12, 14) as u8;
    let funct7i: u8 = bitrange(iword, 25, 31) as u8;
 
    match itype {
        InstType::R => {
            rs1 = Some(RegABI::from_num(rs1i as u8)?);
            rs2 = Some(RegABI::from_num(rs2i as u8)?);
            rd = Some(RegABI::from_num(rdi as u8)?);
            funct3 = Some(funct3i);
            funct7 = Some(funct7i);
        },
        InstType::I => {
            rs1 = Some(RegABI::from_num(rs1i as u8)?);
            rd = Some(RegABI::from_num(rdi as u8)?);
            imm = Some(bitrange(iword, 20, 31));
        },
        InstType::S => {
            rs1 = Some(RegABI::from_num(rs1i as u8)?);
            rs2 = Some(RegABI::from_num(rs2i as u8)?);
            imm = Some((bitrange(iword, 25, 31) << 5) + bitrange(iword, 7, 11));
        },
        InstType::B => {
            rs1 = Some(RegABI::from_num(rs1i as u8)?);
            rs2 = Some(RegABI::from_num(rs2i as u8)?);
            imm = Some((bitrange(iword, 8, 11) << 1)
                + (bitrange(iword, 25, 30) << 5)
                + (bitrange(iword, 7, 7) << 11)
                + (bitrange(iword, 31, 31) << 12));
        }
        InstType::U => {
            imm = Some(bitrange(iword, 12, 31));
            rd = Some(RegABI::from_num(rdi as u8)?);
            funct3 = None;
        },
        InstType::J => {
            rd = Some(RegABI::from_num(rdi as u8)?);
            imm = Some((bitrange(iword, 21, 30) << 1)
            + (bitrange(iword, 20, 20) << 11)
            + (bitrange(iword, 12, 19) << 12)
            + (bitrange(iword, 31, 31) << 20)) ;
        },
        InstType::ExtI(etype) => {
            exten = Some(extended::try_subfields(iword, etype));
        }
        
    }
    Ok(SubFields{ rd , rs1, rs2, funct3, funct7, imm, exten })
    // RS1, RS2, RD, and Funct3 are always in the same place.
    
    
}


// Helper Functions

// This returns a whole word instead of an array of bytes
pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    bytes[0] as u32
    + ((bytes[1] as u32) << 8)
    + ((bytes[2] as u32) << 16)
    + ((bytes[3] as u32) << 24)
}
/// This returns a range of bits, inclusive on both ends, as specified in a datasheet
pub fn bitrange(num: u32, start: usize, end: usize) -> u32 {
    (num >> start) & ((1 << (1 + end - start)) - 1)
}


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