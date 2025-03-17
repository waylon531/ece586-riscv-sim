addi t0, zero, 0xffffffff
addi t1, zero, 0xffff0000
slt t2, t0, t1 ; t2 should be 1 since t0 < t1 (signed)