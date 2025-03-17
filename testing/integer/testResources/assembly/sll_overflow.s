addi t0, zero, 0xffffffff
addi t1, zero, 1
sll t2, t0, t1 ; should result in 0xffff_fffe bc bit gets shifted off the end