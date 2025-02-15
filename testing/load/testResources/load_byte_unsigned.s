addi t0, zero, 0xFF
# Load the unsigned byte from memory at address in t0 into register t1.
# 'lbu' loads a byte and zero-extends it to the register width.
lbu t1, 0(t0)
