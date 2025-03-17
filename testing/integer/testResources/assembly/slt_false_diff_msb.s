addi t2, zero, 1
addi t0, zero, 1
addi t1, zero, 0xffff0000
slt t2, t0, t1 ; t2 <-- 0 bc t0 !< t1 (signed)