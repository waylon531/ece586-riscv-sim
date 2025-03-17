addi t0, zero, 0xffffffff
addi t1, zero, 1
srl t2, t0, t1 ; should result in 0x7fff_ffff (MSB zero-filled)