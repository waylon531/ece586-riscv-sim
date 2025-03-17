addi t0, zero, -10
addi t1, zero, -6
add t2, t1, t0 ; case analysis 2: operands < 0, same sign bit.
               ; expected register state: t0 <-- 0xffff_fff6, t1 <-- 0xffff_fffa, t2 <-- 0xffff_fff0