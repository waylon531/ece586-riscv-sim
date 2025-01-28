use crate::register::Register;

// I'm not sure where sign extension should happen, but it's probably fine to do it in the VM
// Maybe there could be different types of immediates here depending on the size?
// Which instructions sign-extend the immediate?
type Immediate = usize;

enum Operation {
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
