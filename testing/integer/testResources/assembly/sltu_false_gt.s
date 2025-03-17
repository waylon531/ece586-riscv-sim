addi t2, zero, 1 ; start t2 with nonzero value
addi t0, zero, 2
addi t1, zero, 1
sltu t2, t0, t1 ; t2 should be 0 since t0 !< t1 (unsigned)