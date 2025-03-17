addi t0, zero, 0xffff0000
addi t1, zero, 1
slt t2, t0, t1 ; t2 <-- 1 bc t0 < t1 (signed)