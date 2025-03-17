addi t0, zero, 1
addi t1, zero, 0xffff0000
sltu t2, t0, t1 ; t2 <-- 1 bc t0 < t1 (unsigned)