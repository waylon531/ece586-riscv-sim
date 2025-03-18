.global main
main:
    li t1, 0xFA
    la t2, my_val
    sb t1, (t2)
    lbu t3, (t2)
    ret

.data
    my_val: .word 0
