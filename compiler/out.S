.globl _start

_start:
mv x12, x10
li x15, 0
li x10, 0
li x16, 32
sra x14, x11, x15
andi x14, x14, 1
sll x13, x12, x15
addi x15, x15, 1
beq x14, x0, 40
add x10, x10, x13
bne x15, x16, 16
ret
li x1, 0
li x2, 0
li x3, 0
li x4, 0
li x5, 0
li x6, 0
li x7, 0
li x8, 0
li x9, 0
li x11, 0
li x12, 0
li x13, 0
li x14, 0
li x15, 0
li x16, 0
li x17, 0
li x18, 0
li x19, 0
li x20, 0
li x21, 0
li x22, 0
li x23, 0
li x24, 0
li x25, 0
li x26, 0
li x27, 0
li x28, 0
li x29, 0
li x30, 0
li x31, 0
lui x2, 3
addi x2, x2, 2044
sll x2, x2, x10
jal 332
li x10, 0
jal 272
j 192
nop
nop
nop
nop
addi x2, x2, -32
sw x1, 28(x2)
sw x8, 24(x2)
addi x8, x2, 32
sw x10, -20(x8)
li x15, 512
lw x15, 0(x15)
lw x14, -20(x8)
sw x14, 0(x15)
lw x15, -20(x8)
mv x10, x15
lw x1, 28(x2)
lw x8, 24(x2)
addi x2, x2, 32
ret
addi x2, x2, -32
sw x1, 28(x2)
sw x8, 24(x2)
addi x8, x2, 32
sw x10, -20(x8)
li x15, 516
lw x15, 0(x15)
lw x14, -20(x8)
sw x14, 0(x15)
lw x15, -20(x8)
mv x10, x15
lw x1, 28(x2)
lw x8, 24(x2)
addi x2, x2, 32
ret
addi x2, x2, -16
sw x1, 12(x2)
sw x8, 8(x2)
sw x9, 4(x2)
li x12, 0
li x15, 0
li x16, 3
li x10, 2
li x11, 32
sra x14, x16, x15
andi x14, x14, 1
sll x13, x10, x15
addi x15, x15, 1
beq x14, x0, 392
add x12, x12, x13
bne x15, x11, 368
li x15, 6
auipc x8, 0
addi x8, x8, 96
auipc x9, 0
addi x9, x9, 102
beq x12, x15, 468
lbu x10, 0(x8)
addi x8, x8, 1
jal 212
bne x8, x9, 420
li x10, 1
jal 272
lw x1, 12(x2)
lw x8, 8(x2)
lw x9, 4(x2)
li x10, 0
addi x2, x2, 16
ret
li x10, 79
jal 212
li x10, 107
jal 212
li x10, 0
jal 272
j 444