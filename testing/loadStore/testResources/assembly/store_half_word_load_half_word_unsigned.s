addi sp, sp, -2 # grow stack pointer down by 2 bytes
addi fp, sp, 2 # set the frame pointer set the stack pointer to where the FP was
# put two bytes into $t0
addi t0, zero, 0xFF # put first byte in
# rotate it left
slli t0, t0, 8
# or in 11
ori t0, t0, 0x11
sh t0, -2(fp) # store what is at $t0 relative to the fp
lhu t1, -2(fp)
jalr ra
