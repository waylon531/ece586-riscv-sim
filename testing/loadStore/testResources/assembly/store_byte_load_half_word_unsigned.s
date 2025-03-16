addi sp, sp, -1 # grow stack pointer by 1 bytes
addi fp, sp, 1 # set the frame pointer
addi t0, zero, 0xFF # put one byte into $t0
sb t0, -1(fp) 
lhu t1, -1(fp)
jalr ra