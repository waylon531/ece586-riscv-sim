.global main
.data
msg:
  .string "Hello World\n"
  .byte 0
.text
main:
li a0, 1
la a1, msg
la a2, 12
li a7, 64
ecall
jr ra
