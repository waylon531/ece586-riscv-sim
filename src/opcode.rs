use crate::register::Register;
use crate::decode::{InstructionType,ParseError,bytes_to_u32};

// I'm not sure where sign extension should happen, but it's probably fine to do it in the VM
// Maybe there could be different types of immediates here depending on the size?
// Which instructions sign-extend the immediate?
type Immediate = i32;

pub enum Operation {
    // Immediate, register, register instructions
    // RD is first
    ADDI(Register,Register,Immediate),
    SLTI(Register,Register,Immediate),
    SLTIU(Register,Register,Immediate),
    ANDI(Register,Register,Immediate),
    ORI(Register,Register,Immediate),
    XORI(Register,Register,Immediate),
    SLLI(Register,Register,Immediate),
    SRLI(Register,Register,Immediate),
    SRAI(Register,Register,Immediate),
    LUI(Register,Immediate),
    AUIPC(Register,Immediate),

    // Integer, register, register instructions
    // RD first, then SRC1, then SRC2
    ADD(Register,Register,Register),
    SLTU(Register,Register,Register),
    SLT(Register,Register,Register),
    AND(Register,Register,Register),
    OR(Register,Register,Register),
    XOR(Register,Register,Register),
    SLL(Register,Register,Register),
    SRL(Register,Register,Register),
    SUB(Register,Register,Register),
    SRA(Register,Register,Register),
    // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
    NOP,

    // Control transfer instructions
    // Normal, unconditional jumps use x0 as the register
    JAL(Register, Immediate),
    JALR(Register, Register, Immediate),

    // Conditional branches
    BEQ(Register,Register,Immediate),
    BNE(Register,Register,Immediate),
    BLT(Register,Register,Immediate),
    BLTU(Register,Register,Immediate),
    BGE(Register,Register,Immediate),
    BGEU(Register,Register,Immediate),

    // Loads and stores
    LW(Register,Register,Immediate),
    LH(Register,Register,Immediate),
    LHU(Register,Register,Immediate),
    LB(Register,Register,Immediate),
    LBU(Register,Register,Immediate),

    SW(Register,Register,Immediate),
    SH(Register,Register,Immediate),
    SB(Register,Register,Immediate),


    // Evironment call/syscall
    ECALL,

    // Breakpoint for us
    EBREAK,

    // Fence is treated as a NOP
    FENCE,

    // Generic performance hint, we don't need to store any information for them
    // and they are effectively NOPs
    HINT
}
impl Operation {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self,ParseError> {
        use InstructionType::*;
        use Operation::*;
        let combined = bytes_to_u32(bytes);
        // NOTE: Sign extension should happen here for immediates
        
        Ok(match InstructionType::from_bytes(bytes) {
            Ok(RType {
                rd, rs1, rs2, funct3, funct7, opcode
            }) => {
                match opcode {
                    0b0110011 => {
                        // This can be ADD, SUB, SLL, SLT, SLTU, XOR
                        match (funct3, funct7) {
                            (0,0) => ADD(rd,rs1,rs2),
                            (0,0b0100000) => SUB(rd,rs1,rs2),
                            (0b001,0) => SLL(rd,rs1,rs2),
                            (0b010,0) => SLT(rd,rs1,rs2),
                            (0b011,0) => SLTU(rd,rs1,rs2),
                            (0b100,0) => XOR(rd,rs1,rs2),
                            (0b101,0) => SRL(rd,rs1,rs2),
                            (0b101,0b0100000) => SRA(rd,rs1,rs2),
                            (0b110,0) => OR(rd,rs1,rs2),
                            (0b111,0) => AND(rd,rs1,rs2),
                            _ => return Err(ParseError::InvalidInstruction(combined))
                        }
                    },
                    _ => return Err(ParseError::InvalidOpcode(opcode)) // use `op` instead of _?
                }
            },
            Ok(IType {
                rd, rs1, imm, funct3, opcode
            }) => {
                match opcode {
                    // JALR only
                    0b1100111 => {
                        match funct3 {
                            0b000 => JALR(rd,rs1,imm),
                            _ => return Err(ParseError::InvalidInstruction(combined))
                        }
                    },
                    0b0000011 => {
                        match funct3 {
                            0b000 => LB(rd,rs1,imm),
                            0b001 => LH(rd,rs1,imm),
                            0b010 => LW(rd,rs1,imm),
                            0b100 => LBU(rd,rs1,imm),
                            0b101 => LHU(rd,rs1,imm),
                            _ => return Err(ParseError::InvalidInstruction(combined))
                        }
                    },
                    0b0010011 => {
                        match funct3 {
                            0b000 => ADDI(rd,rs1,imm),
                            0b010 => SLTI(rd,rs1,imm),
                            0b011 => SLTIU(rd,rs1,imm),
                            0b100 => XORI(rd,rs1,imm),
                            0b110 => ORI(rd,rs1,imm),
                            0b111 => ANDI(rd,rs1,imm),
                            _ => return Err(ParseError::InvalidInstruction(combined))
                        }
                    },
                    _ => return Err(ParseError::InvalidOpcode(opcode))
                }
            },
            Ok(SType {
                rs1, rs2, imm, funct3, opcode
            }) => {
                match opcode {
                    0b0100011 => {
                        match funct3 {
                            0b000 => SB(rs2,rs1,imm),
                            0b001 => SH(rs2,rs1,imm),
                            0b010 => SW(rs2,rs1,imm),
                            _ => return Err(ParseError::InvalidInstruction(combined))
                        }
                    }
                    _ => return Err(ParseError::InvalidInstruction(combined))
                }
            }
            _ => {unimplemented!()}



        })

    }

}
