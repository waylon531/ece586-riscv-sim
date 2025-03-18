.global main
.data
filename:
  .string "/tmp/test.out"
msg:
  .string "Hello World"
output:
  .space 12
.text
main:
li a7, 56
la a0, filename
li a1, 0x40
ecall
li a7, 64
la a1, msg
li a2, 12
ecall
li a7, 63
la a1, output
ecall
li a0, 1
li a7, 64
ecall
jr ra
