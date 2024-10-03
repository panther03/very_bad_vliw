	.file	"mul.c"
	.option nopic
	.attribute arch, "rv32i2p1"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.align	2
	.globl	multiply
	.type	multiply, @function
multiply:
	mv	a2,a0
	li	a5,0
	li	a0,0
	li	a6,32
.L3:
	sra	a4,a1,a5
	andi	a4,a4,1
	sll	a3,a2,a5
	addi	a5,a5,1
	beq	a4,zero,.L2
	add	a0,a0,a3
.L2:
	bne	a5,a6,.L3
	ret
	.size	multiply, .-multiply
	.section	.rodata.str1.4,"aMS",@progbits,1
	.align	2
.LC0:
	.string	"Jello, world!\n"
	.section	.text.startup,"ax",@progbits
	.align	2
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-16
	sw	s1,4(sp)
	sw	ra,12(sp)
	sw	s0,8(sp)
	sw	s2,0(sp)
	li	s1,0
	li	a5,0
	li	a1,69
	li	a3,32
.L11:
	sra	a4,a1,a5
	andi	a4,a4,1
	sll	a2,a3,a5
	addi	a5,a5,1
	beq	a4,zero,.L10
	add	s1,s1,a2
.L10:
	bne	a5,a3,.L11
	li	a5,4096
	addi	a5,a5,-1888
	lla	s0,.LC0
	lla	s2,.LC0+14
	beq	s1,a5,.L22
.L12:
	lbu	a0,0(s0)
	addi	s0,s0,1
	call	putchar
	bne	s0,s2,.L12
	addi	a0,s1,48
	call	putchar
	li	a0,1
	call	exit
.L13:
	lw	ra,12(sp)
	lw	s0,8(sp)
	lw	s1,4(sp)
	lw	s2,0(sp)
	li	a0,0
	addi	sp,sp,16
	jr	ra
.L22:
	li	a0,79
	call	putchar
	li	a0,107
	call	putchar
	li	a0,0
	call	exit
	j	.L13
	.size	main, .-main
	.ident	"GCC: (Arch Linux Repositories) 14.1.0"
	.section	.note.GNU-stack,"",@progbits
