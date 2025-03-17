use std::{default, ops::Sub};

use crate::Error;
use extended::ExtOpcode;
pub use parse::ParseError;
use super::regfile::*;

mod parse;
mod extended;

#[derive(Debug)]
pub struct Instruction {
    inst: Inst,
    opcode: Opcode,
    inst_type: InstType,
    subfields: SubFields
}
impl Instruction {
    pub fn from_bytes(instruction_bytes: &[u8]) -> Result<Instruction,ParseError> {
        // opcode is 7 bits; this masks off the MSB
        let opcode_bits = bytes[0] & 0x7F;

        let opcode = Opcode::new(opcode_bits)?;

        let inst_type =  InstType::new(opcode);

        let sub_fields=SubFields::new(inst_type, instruction_bytes)?;

        let inst  = Inst::new()
        
        Ok(Instruction {
            inst: inst,
            opcode: Opcode::new(opcode_bits)?,
            inst_type: inst_type

        })
    }
}



#[derive(Debug)]
pub enum Inst {
    LUI,
    AUIPC,
    JAL,
    JALR,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    ADDI,
    SLTI,
    SLTIU,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRAI,
    ADD,
    SUB,
    SLL,
    SLT,
    SLTU,
    XOR,
    SRL,
    SRA,
    OR,
    AND,
    FENCE,
    HINT,
    NOP,
    ECALL,
    ExtA(extended::ExtInst)
}
impl Inst {
    pub fn new(opcode: Opcode, sub_fields: SubFields) -> Result<Opcode,ParseError> {
        use Opcode::*;
        let o = match opcode {
            OP => {
                match (sub_fields.funct3,sub_fields.funct7) {
                    (0x0,0x0) => Inst::ADD,
                    (0x0,0x20) => Inst::SUB,
                    (0x4,0x0) => Inst::XOR,
                    
                    _ => ParseError::InvalidInstruction((0))
                }
            },
            OPIMM => {

            },
            LOAD => {

            },
            STORE => {

            },
            BRANCH => {

            },
            _ => ParseError::InvalidInstruction((0))
        };
        o?
    }
}
#[derive(Debug)]
pub enum Opcode {
    OP,
    OPIMM,
    LOAD,
    STORE,
    BRANCH,
    JAL,
    JALR,
    LUI,
    AUIPC,
    ECALL,
    ExtO(extended::ExtOpcode)

}
impl Opcode {
    pub fn new(opcode: u8) -> Result<Opcode,ParseError>  {
        use Opcode::*;
        Ok(match opcode {
            0b0110011 => OP,
            0b0010011 => OPIMM,
            0b0000011 => LOAD,
            0b0100011 => STORE,
            0b1100011 => BRANCH,
            0b1101111 => JAL,
            0b1100111 => JALR,
            0b0110111 => LUI,
            0b0010111 => AUIPC,
            0b1110011 => ECALL,
            _ => ExtO(extended::try_opcode(opcode)?)
        })
    }
}

#[derive(Debug)]
pub enum InstType {
    R,
    I,
    S,
    B,
    U,
    J,
    ExtI(extended::ExtInsttype)
}




impl InstType {
    pub fn new(opcode: Opcode) -> Self {
        use Opcode::*;
        use InstType::*;
        match opcode {
            OP | JAL=> R,
            OPIMM | ECALL | LOAD | JALR => I,
            STORE => S,
            BRANCH => B,
            LUI | AUIPC => U,
            ExtO(ex) => ExtI(extended::try_insttype(ex))
        }

    }
}
#[derive(Debug)]
pub struct SubFields {
    rd: Option<RegABI>,
    rs1: Option<RegABI>,
    rs2: Option<RegABI>,
    funct3: Option<u8>,
    funct7: Option<u8>,
    imm: Option<u32>,
    exten: Option<extended::ExtSubfields>
}

impl SubFields {
    pub fn new(itype: InstType, instruction_bytes:&[u8]) -> Result<SubFields,ParseError> {
        let instruction_u32 = parse::bytes_to_u32(instruction_bytes);
        
        Ok(parse::get_subfields(instruction_u32, itype))?

    }
}

/*impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Operation::*;
        match self {
            ADDI(r1, r2, imm) =>
                write!(f,"ADDI  {r1}, {r2}, {imm:#x}"),
            SLTI(r1, r2, imm) =>
                write!(f,"SLTI  {r1}, {r2}, {imm:#x}"),
            SLTIU(r1, r2, imm) =>
                write!(f,"SLTIU {r1}, {r2}, {imm:#x}"),
            ANDI(r1, r2, imm) => 
                write!(f,"ANDI  {r1}, {r2}, {imm:#x}"),
            ORI(r1, r2, imm) => 
                write!(f,"ORI   {r1}, {r2}, {imm:#x}"),
            XORI(r1, r2, imm) => 
                write!(f,"ORI   {r1}, {r2}, {imm:#x}"),
            SLLI(r1, r2, imm) => 
                write!(f,"SLLI  {r1}, {r2}, {imm:#x}"),
            SRLI(r1, r2, imm) => 
                write!(f,"SRLI  {r1}, {r2}, {imm:#x}"),
            SRAI(r1, r2, imm) => 
                write!(f,"SRAI  {r1}, {r2}, {imm:#x}"),
            LUI(r1, imm) => 
                write!(f,"LUI   {r1}, {imm:#x}"),
            AUIPC(r1, imm) => 
                write!(f,"AUIPC {r1}, {imm:#x}"),


            // Integer, register, register instructions
            // RD first, then SRC1, then SRC2
            ADD(r1, r2, r3) => 
                write!(f,"ADD   {r1}, {r2}, {r3}"),
            SLTU(r1, r2, r3) => 
                write!(f,"SLTU  {r1}, {r2}, {r3}"),
            SLT(r1, r2, r3) => 
                write!(f,"SLT   {r1}, {r2}, {r3}"),
            AND(r1, r2, r3) => 
                write!(f,"AND   {r1}, {r2}, {r3}"),
            OR(r1, r2, r3) => 
                write!(f,"OR    {r1}, {r2}, {r3}"),
            XOR(r1, r2, r3) => 
                write!(f,"XOR   {r1}, {r2}, {r3}"),
            SLL(r1, r2, r3) => 
                write!(f,"SLL   {r1}, {r2}, {r3}"),
            SRL(r1, r2, r3) => 
                write!(f,"SRL   {r1}, {r2}, {r3}"),
            SUB(r1, r2, r3) => 
                write!(f,"SUB   {r1}, {r2}, {r3}"),
            SRA(r1, r2, r3) => 
                write!(f,"SRA   {r1}, {r2}, {r3}"),

            // Control transfer instructions
            // Normal, unconditional jumps use x0 as the register
            JAL(r1, imm) => 
                write!(f,"JAL   {r1}, {imm:#x}"),
            JALR(r1, r2, imm) => 
                write!(f,"JALR  {r1}, {r2}, {imm:#x}"),

            // Conditional branches
            // Which register is first, rs1 or rs2?
            BEQ(r1, r2, imm) => 
                write!(f,"BEQ   {r1}, {r2}, {imm:#x}"),
            BNE(r1, r2, imm) => 
                write!(f,"BNE   {r1}, {r2}, {imm:#x}"),
            BLT(r1, r2, imm) => 
                write!(f,"BLT   {r1}, {r2}, {imm:#x}"),
            BLTU(r1, r2, imm) => 
                write!(f,"BLTU  {r1}, {r2}, {imm:#x}"),
            BGE(r1, r2, imm) => 
                write!(f,"BGE   {r1}, {r2}, {imm:#x}"),
            BGEU(r1, r2, imm) => 
                write!(f,"BGEU  {r1}, {r2}, {imm:#x}"),

            // Loads and stores
            LW(r1, r2, imm) => 
                write!(f,"LW    {r1}, {r2}, {imm:#x}"),
            LH(r1, r2, imm) => 
                write!(f,"LH    {r1}, {r2}, {imm:#x}"),
            LHU(r1, r2, imm) => 
                write!(f,"LHU   {r1}, {r2}, {imm:#x}"),
            LB(r1, r2, imm) => 
                write!(f,"LB    {r1}, {r2}, {imm:#x}"),
            LBU(r1, r2, imm) => 
                write!(f,"LBU   {r1}, {r2}, {imm:#x}"),

            SW(r1, r2, imm) => 
                write!(f,"SW    {r1}, {r2}, {imm:#x}"),
            SH(r1, r2, imm) => 
                write!(f,"SH    {r1}, {r2}, {imm:#x}"),
            SB(r1, r2, imm) => 
                write!(f,"SB    {r1}, {r2}, {imm:#x}"),

            _ => write!(f, "{:?}", self)

        }
    } */
