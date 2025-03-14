addi sp, sp, -1 # grow stack pointer by 1 bytes
addi s0, sp, 1 # set the frame pointer
addi t0, zero, 0xFF # put one byte into $t0
sb t0, -1(s0) 
lbu t1, -1(s0)
jalr ra
