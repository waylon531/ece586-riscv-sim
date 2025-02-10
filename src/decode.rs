use crate::register::Register;
use thiserror::Error;

pub enum InstructionType {
    RType {
        rd: Register,
        rs1: Register,
        rs2: Register,
        funct3: u8,
        funct7: u8,
        opcode: u8,
    },
    IType {
        rd: Register,
        rs1: Register,
        imm: u32,
        funct3: u8,
        opcode: u8,
    },
    SType {
        rs1: Register,
        rs2: Register,
        imm: u32,
        funct3: u8,
        opcode: u8,
    },
    BType {
        rs1: Register,
        rs2: Register,
        imm: u32,
        funct3: u8,
        opcode: u8,
    },
    UType {
        rd: Register,
        imm: u32,
        opcode: u8,
    },
    JType {
        rd: Register,
        imm: u32,
        opcode: u8,
    },
}

impl InstructionType {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        // opcode is 7 bits
        let opcode = bytes[0] & 0x7F;
        let combined = bytes_to_u32(bytes);
        // Some of these have to look at both opcode and func3
        match opcode {
            // List out all RType opcodes here, separated by |
            0b0110011 => Ok(InstructionType::RType {
                rd: Register::from_num(bitrange(combined, 7, 11))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 7, 11)))?,
                rs1: Register::from_num(bitrange(combined, 15, 19))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 15, 19)))?,
                rs2: Register::from_num(bitrange(combined, 20, 24))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 20, 24)))?,
                funct3: bitrange(combined, 12, 14) as u8,
                funct7: bitrange(combined, 25, 31) as u8,
                opcode,
            }),
            // List out all IType opcodes here, separated by |
            0b0000011 | 0b0010011 => {
                Ok(InstructionType::IType {
                    rd: Register::from_num(bitrange(combined, 7, 11))
                        .ok_or(ParseError::RegisterDecode(bitrange(combined, 7, 11)))?,
                    rs1: Register::from_num(bitrange(combined, 15, 19))
                        .ok_or(ParseError::RegisterDecode(bitrange(combined, 15, 19)))?,
                    //NOTE: We don't sign extend here because it would mess up
                    //      the SLLI/SRLI/SRAI instructions
                    imm: bitrange(combined, 20, 31),
                    funct3: bitrange(combined, 12, 14) as u8,
                    opcode,
                })
            }
            // List out all SType opcodes here, separated by |
            0b0100011 => Ok(InstructionType::SType {
                rs1: Register::from_num(bitrange(combined, 15, 19))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 15, 19)))?,
                rs2: Register::from_num(bitrange(combined, 20, 24))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 20, 24)))?,
                imm: (bitrange(combined, 25, 31) << 5) + bitrange(combined, 7, 11),
                funct3: bitrange(combined, 12, 14) as u8,
                opcode,
            }),
            // List out all BType opcodes here, separated by |
            0b1100011 => Ok(InstructionType::BType {
                rs1: Register::from_num(bitrange(combined, 15, 19))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 15, 19)))?,
                rs2: Register::from_num(bitrange(combined, 20, 24))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 20, 24)))?,
                imm: bitrange(combined, 8, 11)
                    + (bitrange(combined, 25, 30) << 5)
                    + (bitrange(combined, 7, 7) << 11)
                    + (bitrange(combined, 31, 31) << 12),
                funct3: bitrange(combined, 12, 14) as u8,
                opcode,
            }),
            // List out all UType opcodes here, separated by |
            0b0110111 | 0b0010111 => Ok(InstructionType::UType {
                rd: Register::from_num(bitrange(combined, 7, 11))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 7, 11)))?,
                imm: bitrange(combined, 12, 31),
                opcode,
            }),
            // List out all JType opcodes here, separated by |
            0b1101111 => Ok(InstructionType::JType {
                rd: Register::from_num(bitrange(combined, 7, 11))
                    .ok_or(ParseError::RegisterDecode(bitrange(combined, 7, 11)))?,
                imm: bitrange(combined, 21, 30)
                    + (bitrange(combined, 20, 20) << 11)
                    + (bitrange(combined, 12, 19) << 12)
                    + (bitrange(combined, 31, 31) << 20),
                opcode,
            }),
            op => Err(ParseError::InvalidOpcode(op)),
        }
    }
}
//NOTE: This assumes 4 bytes are passed in. Panic will happen with less
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

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid register: {0}")]
    RegisterDecode(u32),
    #[error("Invalid instruction: {0:#x}")]
    InvalidInstruction(u32),
    #[error("Invalid opcode: {0:#x}")]
    InvalidOpcode(u8),
    #[error("Invalid format: {0:#x}")] // not sure if correct formatting
    InvalidFormat(u32),
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_bitrange() {
        assert_eq!(0xAD, bitrange(0xDEADBEEF, 16, 23));
    }
    #[test]
    fn test_bytes_to_u32() {
        assert_eq!(0xDEADBEEF, bytes_to_u32(&[0xEF, 0xBE, 0xAD, 0xDE]));
    }
}
