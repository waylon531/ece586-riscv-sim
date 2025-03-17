addi t0, zero, 8
addi t1, zero, 1
sra t2, t0, t1 ; should result in 4 (8 / 2, sra = srl when msb is 0)