fn execute(){
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

        // Division


        // Evironment call/syscall
        ECALL => {
            /* Fun with system calls! I think this is technically a BIOS? */

            // a7: syscall, [a0-a5]: arguments
            match self.env.syscall(self.registers[Register::A7], [self.registers[Register::A0],self.registers[Register::A1],self.registers[Register::A2],self.registers[Register::A3], self.registers[Register::A4],self.registers[Register::A5]], &mut self.memory) {
                Ok(result) => { self.set_reg(Register::A0, result as u32); },
                Err(e) => { return Err(e) }
            };
            
        },

        // Breakpoint for us
        EBREAK => return Err(ExecutionError::Breakpoint(self.pc)),

        // Does this actually need an opcode? It's the same as ADDI zero, zero, 0
        NOP => {}

        // Generic performance hint, we don't need to store any information for them
        // and they are effectively NOPs
        // Same with FENCE
        HINT | FENCE => {}
    }
}