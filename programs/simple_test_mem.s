.global main
main:
    li t1,0xDEADBEEF
    sw t1, 0x100(zero)
    sh t1, 0x104(zero)
    srli t1, t1, 16 
    sh t1, 0x106(zero)

    lw t1, 0x104(zero)
    lhu t5, 0x100(zero)
    lhu t6, 0x102(zero)
    slli t6, t6, 16
    add t2, t5, t6
    sub a0, t1, t2
    jr ra
