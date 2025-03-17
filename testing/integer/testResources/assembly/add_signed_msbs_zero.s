addi t0, zero, 10
addi t1, zero, 6
add t2, t1, t0 ; case analysis 1: operands > 0, same sign bit. 
               ; expected register state: t0 <-- 0x0000_000A, t1 <-- 0x0000_0006, t2 <-- 0x0000_0010