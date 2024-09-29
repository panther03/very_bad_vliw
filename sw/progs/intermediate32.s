	.file	"xor.c"
	.option nopic
	.attribute arch, "rv32i2p1"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.section	.text.startup,"ax",@progbits
	.align	2
	.globl	main
	.type	main, @function
main:
	li	a0,76
	ret
	.size	main, .-main
	.ident	"GCC: (Arch Linux Repositories) 14.1.0"
	.section	.note.GNU-stack,"",@progbits
