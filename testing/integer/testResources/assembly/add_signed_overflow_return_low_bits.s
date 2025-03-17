addi t2, zero, 1 ; begin with nonzero value in t2
addi t0, zero, 0xffffffff
addi t1, zero, 0x1
add t2, t1, t0 ; t2 should be zero bc overflow is supposed to return low 32 bits