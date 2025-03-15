# LUI
- `R[rd] = {imm,12'b0}`
- sign extending
- make sure the sign extended value ends up in the destination register
- make sure there are 12'b0x0 in the LSBs and the value we want is in the MSB

# AUIPC
- `R[rd] = PC + {imm, 12'b0}`
- program counter should be in an expected state, run PC, probably just start at 
  0x0
- Also set some nops so PC is at a different value for another test
- set sign extended concatenated value {imm,12'b0} to some known reg to check it 
  at the end of the program

# JAL
- `R[rd] = PC + 4; PC = PC + {imm, 1'b0}`
- This one has the funny immediate encoding, verify that we got it right. Probably
  need to use AUIPC (heh, this also may verify AUIPC too) with a 0 immediate
  field to write to a reg and then check the reg
- check RD is correct

# JALR
- `R[rd] = PC + 4; PC = R[rs1] + imm`
- Verify immediate using AUIPC
- check RD is correct

# Branches
- `PC = PC + {imm, 1'b0}`
- `R[rs1] ? R[rs2]`, where ? is ==, >=, >= unsigned, <, < unsigned, and !=
- Branch instructions: BEQ, BNE, BLT, BGE, BLTU, BGEU
- All immediate fields are the same, verify they are stiched together correctly
  and as always, sign extended. Probably using AUIPC with immedate as 0
- RS1 = LHS, RS2 = RHS.
- Check both branch not taken and taken using AUIPC
- make sure that logic such as 2>=2 is taken 
- run tests with negative numbers and 0 == 0, and 0>=0, etc..

# Loads
- LB, LH, LW, LBU, LHU
- Need to check that immediate is formed properly and signed. No direct way though.
- flash a byte, halfword, and word to memory.
- If the value from mem is unsigned, make sure that the value we flashed is zero extended correctly
- If the value from mem is signed make sure we are sign extending it correctly

# Stores
- SB, SH, SW
- Well same idea as load
- in a reg put a unsigned, 0 extended number (byte, halfword, and word), store it.
  Load byte that we stored into a reg using matching uload, check value.
- in a reg put a signed, 0 extended number (byte, halfword, and word), store it.
  Load byte that we stored into a reg using matching load, check value.

# Immediate Integer Ops
- ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI
- Pretty straight forward.
- Negative immediate, positive immediate, negative value in RS1 and positive value in RS1
- When op is done, print value and compare

# Register Integer Ops
- ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND

# FENCE

# FENCE.TSO

# PAUSE

# ECALL

# EBREAK
