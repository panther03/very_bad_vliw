.word 0x0
.word 0x0
addi x1, x0, 1
lui x4, 0xF000F

sw x2, 0(x0)
a:
.word 0x0
addi x1, x1, -1
addi x2, x2, 1

lw x3, 0(x0)
bge x1, x0, a // x1 initial value + 1 iterations
ori x4, x4, 0x7FC
.word 0x0

// stop program
sw x3, 0x7FC(x4)
.word 0x0
.word 0x0
.word 0x0
