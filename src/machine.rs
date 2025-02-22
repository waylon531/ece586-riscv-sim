use crate::decode::ParseError;
use crate::opcode::Operation;
use crate::register::Register;

use std::fmt::Write;

use thiserror::Error;

pub struct Machine {
    // Maybe this should be on the heap
    // How is memory mapped? Is there max 64K? Or is that just the size of program
    // we can load?
    //
    // NOTE: This is a boxed slice, while it could well be a Vec for simplicity
    // we also really don't want to have someone resizing the mmap
    memory: Box<[u8]>,
    // The top of memory, points right above the last usable address
    memory_top: u32,
    // Store x1-x31
    // x0 is always 0, no reason to store
    registers: [u32; 31],
    pc: u32,
}
impl Machine {
    pub fn new(starting_addr: u32, stack_addr: u32, memory_top: u32, memmap: Box<[u8]>) -> Self{
        let mut m = Machine {
                    memory: memmap,
                    registers: [0;31],
                    memory_top,
                    pc: starting_addr
        };
        m.set_reg(Register::SP, stack_addr);
        m
    }
    // String formatting should never fail, it's likely safe to unwrap here
    pub fn display_info(&self) -> String {
        let mut buf = String::new();
        write!(buf,"PC: {:#x}", self.pc).unwrap();
        for i in 0 .. 31 {
            write!(buf,"\n\rX{1}: {0} {0:#x}",self.registers[i],i+1).unwrap();
        }
        // TODO: Print a little bit of memory context, around where the stack is
        // And some instruction context as well

        buf

    }
    pub fn set_reg(&mut self,reg: Register, value: u32) {
        let reg_num = reg.to_num();
        // Writes to the zero register are NOPs
        if reg_num == 0 {
            return;
        } else {
            self.registers[reg_num - 1] = value;
        }
    }

    // These 4 functions could probably be more modular ...
    pub fn read_instruction_bytes(&self, addr: u32) -> Result<&[u8], ExecutionError> {
        // Error out if the address is not aligned on a 32-bit boundary
        if addr & 0x11 != 0 {
            Err(ExecutionError::InstructionAddressMisaligned(addr))
        // If the memory top is zero then assume we are using the full 4GB address space as memory
        } else if self.memory_top == 0 || addr.overflowing_add(4).0 <= self.memory_top {
            Ok(&self.memory[addr as usize .. addr as usize + 4])
        } else {
            Err(ExecutionError::InstructionAccessFault(addr))
        }
    }
    pub fn read_byte(&self, addr: u32) -> Result<u8, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr < self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize])
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_word(&self, addr: u32) -> Result<u32, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as u32
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u32) << 8)
                + ((self.memory[addr.overflowing_add(2).0 as usize] as u32) << 16)
                + ((self.memory[addr.overflowing_add(3).0 as usize] as u32) << 24))
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_halfword(&self, addr: u32) -> Result<u16, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as u16
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u16) << 8))
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_byte(&mut self, data: u8, addr: u32) -> Result<(), ExecutionError> {
        if addr < self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_halfword(&mut self, data: u16, addr: u32) -> Result<(), ExecutionError> {
        if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }

    pub fn store_word(&mut self, data: u32, addr: u32) -> Result<(), ExecutionError> {
        if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            self.memory[addr.overflowing_add(2).0 as usize] = (data >> 16) as u8;
            self.memory[addr.overflowing_add(3).0 as usize] = (data >> 24) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    // Fetch, decode, and execute an instruction
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        use Operation::*;
        //Fetch and decode
        let op = Operation::from_bytes(self.read_instruction_bytes(self.pc)?)?;

        // Branches and jumps will set this to false
        let mut increment_pc = true;

        match op {
            ADDI(rd, rs1, imm) => {
                self.set_reg(rd, self.registers[rs1].overflowing_add(imm as u32).0)
            }
            SLTI(rd, rs1, imm) => self.set_reg(
                rd,
                if (self.registers[rs1] as i32) < imm {
                    1
                } else {
                    0
                },
            ),
            SLTIU(rd, rs1, imm) => self.set_reg(
                rd,
                if self.registers[rs1] < (imm as u32) {
                    1
                } else {
                    0
                },
            ),
            ANDI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] & (imm as u32)),
            ORI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] | (imm as u32)),
            XORI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] ^ (imm as u32)),
            SLLI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] << (imm as u32)),
            SRLI(rd, rs1, imm) => self.set_reg(rd, self.registers[rs1] >> (imm as u32)),
            // Arithmetic shifts are the default on signed numbers, so convert types a few times
            SRAI(rd, rs1, imm) => {
                self.set_reg(rd, ((self.registers[rs1] as i32) >> (imm as u32)) as u32)
            }

            // NOTE: It is assumed that during decoding, imm is shifted 12 bits over and as such
            // is already in the right format here
            LUI(rd, imm) => self.set_reg(rd, imm as u32),
            AUIPC(rd, imm) => self.set_reg(rd, self.pc.overflowing_add(imm as u32).0),

            // Integer, register, register instructions
            // RD first, then SRC1, then SRC2
            ADD(rd, rs1, rs2) => self.set_reg(
                rd,
                self.registers[rs1].overflowing_add(self.registers[rs2]).0,
            ),
            SLTU(rd, rs1, rs2) => self.set_reg(
                rd,
                if self.registers[rs1] < self.registers[rs2] {
                    1
                } else {
                    0
                },
            ),
            SLT(rd, rs1, rs2) => self.set_reg(
                rd,
                if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {
                    1
                } else {
                    0
                },
            ),
            AND(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] & self.registers[rs2]),
            OR(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] | self.registers[rs2]),
            XOR(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] ^ self.registers[rs2]),
            SLL(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] << self.registers[rs2]),
            SRL(rd, rs1, rs2) => self.set_reg(rd, self.registers[rs1] >> self.registers[rs2]),
            SUB(rd, rs1, rs2) => self.set_reg(
                rd,
                self.registers[rs1].overflowing_sub(self.registers[rs2]).0,
            ),
            SRA(rd, rs1, rs2) => self.set_reg(
                rd,
                ((self.registers[rs1] as i32) >> self.registers[rs2]) as u32,
            ),

            // Control transfer instructions
            // Normal, unconditional jumps use x0 as the register
            JAL(rd, imm) => {
                self.set_reg(rd, self.pc.overflowing_add(4).0);
                // Set the pc, clearing the last bit
                self.pc = self.pc.overflowing_add(imm as u32).0 & (! 0x1); 
                increment_pc = false;
            }
            JALR(rd, rs1, imm) => {
                // A RET to address zero will stop execution
                // The a0 register contains the status code to return
                if rs1 == Register::RA && self.registers[Register::RA] == 0 {
                    return Err(ExecutionError::FinishedExecution(self.registers[Register::A0] as u8))
                }
                self.set_reg(rd,self.pc.overflowing_add(4).0);
                // Add imm to rs1 and zero out lowest bit
                self.pc = self.registers[rs1].overflowing_add(imm as u32).0 & (!1);
                increment_pc = false;
            }

            // Conditional branches
            BEQ(rs1, rs2, imm) => {
                if self.registers[rs1] == self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BNE(rs1, rs2, imm) => {
                if self.registers[rs1] != self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BLT(rs1, rs2, imm) => {
                if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BLTU(rs1, rs2, imm) => {
                if self.registers[rs1] < self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BGE(rs1, rs2, imm) => {
                if (self.registers[rs1] as i32) >= (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }
            BGEU(rs1, rs2, imm) => {
                if self.registers[rs1] >= self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            }

            // Loads and stores
            LW(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_word(self.registers[rs1].overflowing_add_signed(imm).0)?,
            ),
            // There are two casts here, one to sign extend and one to put it into the correct type
            LH(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_halfword(self.registers[rs1].overflowing_add_signed(imm).0)? as i32
                    as u32,
            ),
            LHU(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_halfword(self.registers[rs1].overflowing_add_signed(imm).0)? as u32,
            ),
            LB(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_byte(self.registers[rs1].overflowing_add_signed(imm).0)? as i32 as u32,
            ),
            LBU(rd, rs1, imm) => self.set_reg(
                rd,
                self.read_byte(self.registers[rs1].overflowing_add_signed(imm).0)? as u32,
            ),

            SW(rs1, rs2, imm) => self.store_word(
                self.registers[rs2],
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,
            SH(rs1, rs2, imm) => self.store_halfword(
                (self.registers[rs2] & 0xFFFF) as u16,
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,
            SB(rs1, rs2, imm) => self.store_byte(
                (self.registers[rs2] & 0xFF) as u8,
                self.registers[rs1].overflowing_add_signed(imm).0,
            )?,

            // Evironment call/syscall
            ECALL => {}

            // Breakpoint for us
            EBREAK => return Err(ExecutionError::Breakpoint(self.pc)),

            // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
            NOP => {}

            // Generic performance hint, we don't need to store any information for them
            // and they are effectively NOPs
            // Same with FENCE
            HINT | FENCE => {}
        }

        if increment_pc {
            self.pc = self.pc.overflowing_add(4).0;
        }
        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ExecutionError {
    #[error("Failed to decode instruction: {0}")]
    ParseError(#[from] ParseError),
    #[error("Exception while loading value at address {0:#x}")]
    LoadAccessFault(u32),
    #[error("Exception while loading instruction at address {0:#x}")]
    InstructionAccessFault(u32),
    #[error("Tried to read misaligned instruction at {0:#x}")]
    InstructionAddressMisaligned(u32),
    #[error("Breakpoint hit at address {0:#x}")]
    Breakpoint(u32),
    // This isn't really an error, but it is an exceptional condition
    // maybe could be represented a different way but this is easy
    #[error("Successfully finished execution")]
    FinishedExecution(u8),
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_write_u32() {
        let mut machine = Machine::new(0,0,8,vec![0;4].into_boxed_slice());
        machine.store_word(0xBEE5AA11,0).unwrap();
        for (&mem_value,test_value) in machine.memory.iter().zip([0x11,0xAA,0xE5,0xBE]) {
            assert_eq!(mem_value,test_value);
        }
    }

    #[test]
    fn test_program_completion() {
        let mut machine = Machine::new(0, 0, 32, vec![0; 32].into_boxed_slice());
        let store_a0_42 = 0b0010011 | (Register::A0.to_num() << 7) | (42 << 20);
        let _ = machine.store_word(store_a0_42 as u32,0);
        // JALR to RA
        let ret = 0b1100111 | (Register::RA.to_num() << 15) ;
        let _ = machine.store_word(ret as u32,4);

        machine.step().unwrap();
        assert_eq!(machine.step(),Err(ExecutionError::FinishedExecution(42)))


    }
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn load_store_byte_asm(data: u8, s in 16u32..(1<<11)) {
            let mut machine = Machine::new(0, 0, s+4, vec![0; s as usize+4].into_boxed_slice());
            let store_a0_42: u32 = 0b0010011 | ((Register::T1.to_num()as u32) << 7) | ((data as u32) << 20);
            let _ = machine.store_word(store_a0_42 as u32,0);
            println!("S: {}",s);

            // Store the data byte in s
            let sb: u32 = 0b0100011 | ((Register::T1.to_num()as u32) << 20) 
                | ((s & 0x1F) << 7) | ((s & 0xFE0)<<20);
            let _ = machine.store_word(sb, 4);
            println!("SB: {:08x}",sb);
            println!("TOP: {:08x}",(s & 0xFE0)<<20);

            let lb: u32 = 0b0000011 | ((Register::A0.to_num()as u32) << 7) | (s << 20);
            let _ = machine.store_word(lb,8);

            // JALR to RA
            let ret = 0b1100111 | (Register::RA.to_num() << 15) ;
            let _ = machine.store_word(ret as u32,12);

            for i in 0 .. 4 {
                println!("{:?}",Operation::from_bytes(machine.read_instruction_bytes(i*4)?));
            }
            machine.step().unwrap();
            machine.step().unwrap();
            machine.step().unwrap();
            assert_eq!(machine.step(),Err(ExecutionError::FinishedExecution(data)))
        }
    }

}
