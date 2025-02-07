use crate::decode::ParseError;
use crate::register::Register;
use crate::opcode::Operation;

use thiserror::Error;

struct Machine {
    // Maybe this should be on the heap
    // How is memory mapped? Is there max 64K? Or is that just the size of program
    // we can load?
    memory: [u8; 64*1024],
    // The top of memory, points right above the last usable address
    memory_top: u32,
    // Store x1-x31
    // x0 is always 0, no reason to store
    registers: [u32;31],
    pc: u32
}
impl Machine {
    pub fn new(starting_addr: u32, stack_addr: u32, memory_top: u32) -> Self{
        let mut m = Machine {
                    memory: [0; 64*1024], 
                    registers: [0;31],
                    memory_top,
                    pc: starting_addr
        };
        m.set_reg(Register::SP,stack_addr);
        m

    }
    pub fn set_reg(&mut self,reg: Register, value: u32) {
        let reg_num = reg.to_num();
        // Writes to the zero register are NOPs
        if reg_num == 0 {
            return
        } else {
            self.registers[reg_num - 1] = value;
        }

    }
    
    // These 4 functions could probably be more modular ...
    pub fn read_instruction_bytes(&self, addr: u32) -> Result<&[u8],ExecutionError> {
        // Error out if the address is not aligned on a 32-bit boundary
        if addr & 0x11 != 0{
            Err(ExecutionError::InstructionAddressMisaligned(addr))
        // If the memory top is zero then assume we are using the full 4GB address space as memory
        } else if self.memory_top == 0 || addr.overflowing_add(4).0 <= self.memory_top {
            Ok(&self.memory[addr as usize .. addr as usize + 3])
        } else {
            Err(ExecutionError::InstructionAccessFault(addr))
        }
    }
    pub fn read_byte(&self, addr: u32) -> Result<u8,ExecutionError>{
        //TODO in future: check if this is a memory mapped device
        if addr < self.memory_top {
            Ok(self.memory[addr as usize])
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
        
    }
    pub fn read_word(&self, addr: u32) -> Result<u32, ExecutionError>{
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(4) <= self.memory_top {
            Ok(
                self.memory[addr as usize] as u32 
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u32) << 8)
                + ((self.memory[addr.overflowing_add(2).0 as usize] as u32) << 16)
                + ((self.memory[addr.overflowing_add(3).0 as usize] as u32) << 24)
                )
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }

    }
    pub fn read_halfword(&self, addr: u32) -> Result<u16, ExecutionError>{
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(2) <= self.memory_top {
            Ok(
                self.memory[addr as usize] as u16 
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u16) << 8)
                )
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }

    }
    pub fn store_byte(&mut self, data: u8, addr: u32) -> Result<(),ExecutionError> {
        if addr < self.memory_top {
            self.memory[addr as usize] = data;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
        
    }
    pub fn store_halfword(&mut self, data: u16, addr: u32) -> Result<(),ExecutionError> {
        if addr.saturating_add(2) <= self.memory_top {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }

    pub fn store_word(&mut self, data: u32, addr: u32) -> Result<(),ExecutionError> {
        if addr.saturating_add(4) <= self.memory_top {
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
    pub fn step(&mut self) -> Result<(),ExecutionError> {
        use Operation::*;
        //Fetch and decode
        let op = Operation::from_bytes(self.read_instruction_bytes(self.pc)?)?;

        // Branches and jumps will set this to false
        let mut increment_pc = true;

        match op {
            ADDI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1].overflowing_add(imm as u32).0)},
            SLTI(rd,rs1,imm) => {
                self.set_reg(rd,if (self.registers[rs1] as i32) < imm {1} else {0})
            },
            SLTIU(rd,rs1,imm) => {
                self.set_reg(rd,if self.registers[rs1] < (imm as u32) {1} else {0})
            },
            ANDI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1] & (imm as u32))},
            ORI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1] | (imm as u32))},
            XORI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1] ^ (imm as u32))},
            SLLI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1] << (imm as u32))},
            SRLI(rd,rs1,imm) => {self.set_reg(rd,self.registers[rs1] >> (imm as u32))},
            // Arithmetic shifts are the default on signed numbers, so convert types a few times
            SRAI(rd,rs1,imm) => {self.set_reg(rd,((self.registers[rs1] as i32) >> (imm as u32))as u32)},

            // NOTE: It is assumed that during decoding, imm is shifted 12 bits over and as such
            // is already in the right format here
            LUI(rd,imm) => {self.set_reg(rd, imm as u32)},
            AUIPC(rd,imm) => {self.set_reg(rd,self.pc.overflowing_add(imm as u32).0)},

            // Integer, register, register instructions
            // RD first, then SRC1, then SRC2
            ADD(rd,rs1,rs2) => self.set_reg(rd,self.registers[rs1].overflowing_add(self.registers[rs2]).0),
            SLTU(rd,rs1,rs2) => {
                self.set_reg(rd,if self.registers[rs1] < self.registers[rs2] {1} else {0})
            },
            SLT(rd,rs1,rs2) => {
                self.set_reg(rd,if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {1} else {0})

            },
            AND(rd,rs1,rs2) => self.set_reg(rd, self.registers[rs1] & self.registers[rs2]),
            OR(rd,rs1,rs2) => self.set_reg(rd, self.registers[rs1] | self.registers[rs2]),
            XOR(rd,rs1,rs2) => self.set_reg(rd, self.registers[rs1] ^ self.registers[rs2]),
            SLL(rd,rs1,rs2) => {self.set_reg(rd,self.registers[rs1] << self.registers[rs2])},
            SRL(rd,rs1,rs2) => {self.set_reg(rd,self.registers[rs1] >> self.registers[rs2])},
            SUB(rd,rs1,rs2) => {self.set_reg(rd,self.registers[rs1].overflowing_sub(self.registers[rs2]).0)},
            SRA(rd,rs1,rs2) => {self.set_reg(rd,((self.registers[rs1] as i32) >> self.registers[rs2]) as u32)},

            // Control transfer instructions
            // Normal, unconditional jumps use x0 as the register
            JAL(rd, imm) => {
                self.set_reg(rd,self.pc.overflowing_add(4).0);
                self.pc = self.pc.overflowing_add(imm as u32).0;
                increment_pc = false;
            },
            JALR(rd, rs1, imm) => {
                self.set_reg(rd,self.pc.overflowing_add(4).0);
                // Add imm to rs1 and zero out lowest bit
                self.pc = self.registers[rs1].overflowing_add(imm as u32).0 & (! 1);
                increment_pc = false;
            },

            // Conditional branches
            BEQ(rs1,rs2,imm) => {
                if self.registers[rs1] == self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                }
            },
            BNE(rs1,rs2,imm) =>
                if self.registers[rs1] != self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                },
            BLT(rs1,rs2,imm) => 
                if (self.registers[rs1] as i32) < (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                },
            BLTU(rs1,rs2,imm) => 
                if self.registers[rs1] < self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                },
            BGE(rs1,rs2,imm) => 
                if (self.registers[rs1] as i32) >= (self.registers[rs2] as i32) {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                },
            BGEU(rs1,rs2,imm) => 
                if self.registers[rs1] >= self.registers[rs2] {
                    increment_pc = false;
                    self.pc = self.pc.overflowing_add(imm as u32).0;
                },

            // Loads and stores
            LW(rd,rs1,imm) => self.set_reg(rd,self.read_word(self.registers[rs1].overflowing_add(imm as u32).0)?),
            // There are two casts here, one to sign extend and one to put it into the correct type
            LH(rd,rs1,imm) => self.set_reg(rd,self.read_halfword(self.registers[rs1].overflowing_add(imm as u32).0)? as i32 as u32),
            LHU(rd,rs1,imm) => self.set_reg(rd,self.read_halfword(self.registers[rs1].overflowing_add(imm as u32).0)? as u32),
            LB(rd,rs1,imm) => self.set_reg(rd,self.read_byte(self.registers[rs1].overflowing_add(imm as u32).0)? as i32 as u32),
            LBU(rd,rs1,imm) => self.set_reg(rd,self.read_byte(self.registers[rs1].overflowing_add(imm as u32).0)? as u32),

            SW(rs1, rs2, imm) => self.store_word(self.registers[rs2], self.registers[rs1].overflowing_add_signed(imm).0)?,
            SH(rs1, rs2, imm) => self.store_halfword((self.registers[rs2] & 0xFFFF) as u16, self.registers[rs1].overflowing_add_signed(imm).0)?,
            SB(rs1,rs2,imm) => self.store_byte((self.registers[rs2] & 0xFF) as u8, self.registers[rs1].overflowing_add_signed(imm).0)?,


            // Evironment call/syscall
            ECALL => {},

            // Breakpoint for us
            EBREAK => {},

            // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
            NOP => {},

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

#[derive(Error, Debug)]
enum ExecutionError {
    #[error("Failed to decode instruction: {0}")]
    ParseError(#[from] ParseError),
    #[error("Exception while loading value at address {0:#x}")]
    LoadAccessFault(u32),
    #[error("Exception while loading instruction at address {0:#x}")]
    InstructionAccessFault(u32),
    #[error("Tried to read misaligned instruction at {0:#x}")]
    InstructionAddressMisaligned(u32),
}
