addi sp, sp, -4 # grow stack pointer down by 4 bytes
addi fp, sp, 4 # set the frame pointer set the stack pointer to where the FP was
# put two bytes into $t0
addi t0, zero, 0xFF # put first byte in
# rotate it left
slli t0, t0, 8
# or in second byte
ori t0, t0, 0x11
# rotate it left
slli t0, t0, 8
# or in third byte
ori t0, t0, 0xAE
# rotate it left
slli t0, t0, 8
# or in forth byte
ori t0, t0, 0x89
sw t0, -4(fp) # store what is at $t0 relative to the fp
lw t1, -4(fp)
jalr ra