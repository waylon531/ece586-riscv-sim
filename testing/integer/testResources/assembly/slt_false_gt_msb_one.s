addi t2, zero, 1 ; start t2 with nonzero value
addi t0, zero, 0xffff0000
addi t1, zero, 0xffffffff
slt t2, t0, t1 ; t2 should be 0 since t0 !< t1 (signed)