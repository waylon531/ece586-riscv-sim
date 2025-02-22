use crate::decode::{bytes_to_u32, InstructionType, ParseError, bitrange};
use crate::register::Register;

// I'm not sure where sign extension should happen, but it's probably fine to do it in the VM
// Maybe there could be different types of immediates here depending on the size?
// Which instructions sign-extend the immediate?
type Immediate = i32;

// sign-extend imm field and convert to i32
pub fn sign_extend(num: u32, bits: u8) -> Immediate {
    let shamt: u8 = 32 - bits;
    let res: Immediate = ((num as i32) << shamt) >> shamt;
    res
}

#[derive(Debug)]
pub enum Operation {
    // Immediate, register, register instructions
    // RD is first
    ADDI(Register, Register, Immediate),
    SLTI(Register, Register, Immediate),
    SLTIU(Register, Register, Immediate),
    ANDI(Register, Register, Immediate),
    ORI(Register, Register, Immediate),
    XORI(Register, Register, Immediate),
    SLLI(Register, Register, Immediate),
    SRLI(Register, Register, Immediate),
    SRAI(Register, Register, Immediate),
    LUI(Register, Immediate),
    AUIPC(Register, Immediate),

    // Integer, register, register instructions
    // RD first, then SRC1, then SRC2
    ADD(Register, Register, Register),
    SLTU(Register, Register, Register),
    SLT(Register, Register, Register),
    AND(Register, Register, Register),
    OR(Register, Register, Register),
    XOR(Register, Register, Register),
    SLL(Register, Register, Register),
    SRL(Register, Register, Register),
    SUB(Register, Register, Register),
    SRA(Register, Register, Register),
    // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
    NOP,

    // Control transfer instructions
    // Normal, unconditional jumps use x0 as the register
    JAL(Register, Immediate),
    JALR(Register, Register, Immediate),

    // Conditional branches
    // Which register is first, rs1 or rs2?
    BEQ(Register, Register, Immediate),
    BNE(Register, Register, Immediate),
    BLT(Register, Register, Immediate),
    BLTU(Register, Register, Immediate),
    BGE(Register, Register, Immediate),
    BGEU(Register, Register, Immediate),

    // Loads and stores
    LW(Register, Register, Immediate),
    LH(Register, Register, Immediate),
    LHU(Register, Register, Immediate),
    LB(Register, Register, Immediate),
    LBU(Register, Register, Immediate),

    SW(Register, Register, Immediate),
    SH(Register, Register, Immediate),
    SB(Register, Register, Immediate),

    // Evironment call/syscall
    ECALL,

    // Breakpoint for us
    EBREAK,

    // Fence is treated as a NOP
    FENCE,

    // Generic performance hint, we don't need to store any information for them
    // and they are effectively NOPs
    HINT,
}

impl Operation {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        use InstructionType::*;
        use Operation::*;
        let combined = bytes_to_u32(bytes);
        // NOTE: Sign extension should happen here for immediates
        Ok(match InstructionType::from_bytes(bytes) {
            Ok(RType {
                rd,
                rs1,
                rs2,
                funct3,
                funct7,
                opcode,
            }) => {
                match opcode {
                    0b0110011 => {
                        // This can be ADD, SUB, SLL, SLT, SLTU, XOR
                        match (funct3, funct7) {
                            (0, 0) => ADD(rd, rs1, rs2),
                            (0, 0b0100000) => SUB(rd, rs1, rs2),
                            (0b001, 0) => SLL(rd, rs1, rs2),
                            (0b010, 0) => SLT(rd, rs1, rs2),
                            (0b011, 0) => SLTU(rd, rs1, rs2),
                            (0b100, 0) => XOR(rd, rs1, rs2),
                            (0b101, 0) => SRL(rd, rs1, rs2),
                            (0b101, 0b0100000) => SRA(rd, rs1, rs2),
                            (0b110, 0) => OR(rd, rs1, rs2),
                            (0b111, 0) => AND(rd, rs1, rs2),
                            _ => return Err(ParseError::InvalidInstruction(combined)),
                        }
                    }
                    _ => return Err(ParseError::InvalidOpcode(opcode)), // use `op` instead of _?
                }
            }
            Ok(IType {
                rd,
                rs1,
                imm,
                funct3,
                opcode,
            }) => {
                // note that `imm` (unsigned) is used for SLLI, SR[L|A]I
                // all other instrs sign-extend as per manual
                let imm_s: Immediate = sign_extend(imm, 12);
                match opcode {
                    // JALR only
                    0b1100111 => match funct3 {
                        0b000 => JALR(rd, rs1, imm_s),
                        _ => return Err(ParseError::InvalidInstruction(combined)),
                    },
                    0b0000011 => match funct3 {
                        0b000 => LB(rd, rs1, imm_s),
                        0b001 => LH(rd, rs1, imm_s),
                        0b010 => LW(rd, rs1, imm_s),
                        0b100 => LBU(rd, rs1, imm_s),
                        0b101 => LHU(rd, rs1, imm_s),
                        _ => return Err(ParseError::InvalidInstruction(combined)),
                    },
                    0b0010011 => match funct3 {
                        // this may need refactoring...
                        0b001 | 0b101 => {
                            let shamt = bitrange(imm,0,4) as i32;
                            match funct3 {
                                0b001 => SLLI(rd,rs1,shamt),
                                0b101 => {
                                    let b: i32 = (imm >> 10) as i32;
                                    match b {
                                        0b0 => SRLI(rd,rs1,shamt),
                                        0b1 => SRAI(rd,rs1,shamt),
                                        _ => return Err(ParseError::InvalidInstruction(combined)),
                                    }
                                },
                                _ => return Err(ParseError::InvalidInstruction(combined)),
                            }
                        },
                        0b000 => ADDI(rd, rs1, imm_s),
                        0b010 => SLTI(rd, rs1, imm_s),
                        0b011 => SLTIU(rd, rs1, imm_s),
                        0b100 => XORI(rd, rs1, imm_s),
                        0b110 => ORI(rd, rs1, imm_s),
                        0b111 => ANDI(rd, rs1, imm_s),
                        _ => return Err(ParseError::InvalidInstruction(combined)),
                    },
                    _ => return Err(ParseError::InvalidOpcode(opcode)),
                }
            }
            Ok(SType {
                rs1,
                rs2,
                imm,
                funct3,
                opcode,
            }) => {
                let imm_s: Immediate = sign_extend(imm, 12);
                match opcode {
                    0b0100011 => match funct3 {
                        0b000 => SB(rs1, rs2, imm_s),
                        0b001 => SH(rs1, rs2, imm_s),
                        0b010 => SW(rs1, rs2, imm_s),
                        _ => return Err(ParseError::InvalidInstruction(combined)),
                    },
                    _ => return Err(ParseError::InvalidInstruction(combined)),
                }
            }
            Ok(BType {
                rs1,
                rs2,
                imm,
                funct3,
                opcode,
            }) => {
                let imm_s: Immediate = sign_extend(imm, 12);
                match opcode {
                    0b1100011 => match funct3 {
                        0b000 => BEQ(rs1, rs2, imm_s),
                        0b001 => BNE(rs1, rs2, imm_s),
                        0b100 => BLT(rs1, rs2, imm_s),
                        0b101 => BGE(rs1, rs2, imm_s),
                        0b110 => BLTU(rs1, rs2, imm_s),
                        0b111 => BGEU(rs1, rs2, imm_s),
                        _ => return Err(ParseError::InvalidInstruction(combined)),
                    },
                    _ => return Err(ParseError::InvalidInstruction(combined)),
                }
            }
            Ok(UType { rd, imm, opcode }) => {
                let imm_s: Immediate = sign_extend(imm, 20);
                match opcode {
                    0b0110111 => LUI(rd, imm_s << 12),
                    0b0010111 => AUIPC(rd, imm_s << 12),
                    _ => return Err(ParseError::InvalidInstruction(combined)),
                }
            }
            Ok(JType { rd, imm, opcode }) => {
                // NOTE: The immediate is shifted over by one here and doubled, 
                //       so the 20 bit immediate has its MSB at the 21st position
                let imm_s: Immediate = sign_extend(imm, 21);
                match opcode {
                    0b1101111 => JAL(rd, imm_s),
                    _ => return Err(ParseError::InvalidInstruction(combined)),
                }
            }
            // Bubble the error from opcode parsing up
            Err(e) => return Err(e),
        })
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn sign_extend_test() {
        assert_eq!(sign_extend(0xFF,8),-1);
    }

    #[test]
    fn no_sign_extend_test() {
        assert_eq!(sign_extend(0x7F,8),0x7F);

    }
}
