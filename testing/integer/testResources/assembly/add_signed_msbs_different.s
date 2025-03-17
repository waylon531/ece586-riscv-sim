addi t0, zero, -16
addi t1, zero, 10
add t2, t1, t0 ; case analysis 3: different sign bits. 
               ; expected register state: t0 <-- 0xffff_fff0, t1 <-- 0x0000_000a, t2 <-- 0xffff_fffa