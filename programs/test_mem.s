	.file	"test_mem.c"
	.option pic
	.attribute arch, "rv32i2p1"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.globl	multiplicands
	.data
	.align	2
	.type	multiplicands, @object
	.size	multiplicands, 3
multiplicands:
	.ascii	"\003\004\005"
	.globl	nums
	.align	2
	.type	nums, @object
	.size	nums, 6
nums:
	.half	512
	.half	600
	.half	452
	.globl	result
	.bss
	.align	2
	.type	result, @object
	.size	result, 12
result:
	.zero	12
	.text
	.align	2
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-32
	sw	ra,28(sp)
	sw	s0,24(sp)
	addi	s0,sp,32
	sw	zero,-20(s0)
	sw	zero,-24(s0)
	sw	zero,-28(s0)
	sw	zero,-32(s0)
	sw	zero,-20(s0)
	j	.L2
.L5:
	la	a4,multiplicands
	lw	a5,-20(s0)
	add	a5,a4,a5
	lbu	a5,0(a5)
	sw	a5,-24(s0)
	sw	zero,-32(s0)
	j	.L3
.L4:
	la	a4,nums
	lw	a5,-20(s0)
	slli	a5,a5,1
	add	a5,a4,a5
	lh	a5,0(a5)
	mv	a4,a5
	lw	a5,-32(s0)
	add	a5,a5,a4
	sw	a5,-32(s0)
	lw	a5,-24(s0)
	addi	a5,a5,-1
	sw	a5,-24(s0)
.L3:
	lw	a5,-24(s0)
	bge	a5,zero,.L4
	la	a4,result
	lw	a5,-20(s0)
	slli	a5,a5,2
	add	a5,a4,a5
	lw	a4,-32(s0)
	sw	a4,0(a5)
	lw	a4,-28(s0)
	lw	a5,-32(s0)
	add	a5,a4,a5
	sw	a5,-28(s0)
	lw	a5,-20(s0)
	addi	a5,a5,1
	sw	a5,-20(s0)
.L2:
	lw	a4,-20(s0)
	li	a5,2
	ble	a4,a5,.L5
	lw	a5,-28(s0)
	mv	a0,a5
	lw	ra,28(sp)
	lw	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	main, .-main
	.ident	"GCC: () 14.2.0"
	.section	.note.GNU-stack,"",@progbits
