# grow stack pointer down by 65,536 bytes
addi t4, zero, -1
addi t5, zero, 1
slli t4, t4, 16
slli t5, t5, 16
add sp, sp, t4
add fp, zero, sp # set FP == SP
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
# try to store something outside the allocated memory at 65,537
sw t0, -1(fp) # this should cause an exception
lw t1, -1(fp)
jalr ra
