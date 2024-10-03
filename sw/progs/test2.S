addi x1, x0, 10
lui x4, 0xF000F
a:
sw x2, 0(x0)
addi x1, x1, -1
addi x2, x2, 1
lw x3, 0(x0)
bge x1, x0, a // x1 initial value + 1 iterations
ori x4, x4, 0x7FC
// stop program
sw x3, 0x7FC(x4)

