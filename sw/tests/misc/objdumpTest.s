li      ra,69
li      sp,69
li      gp,69
li      tp,69
li      t0,69
li      t1,69
li      t2,69
li      s0,69
li      s1,69
li      a1,69
li      a2,69
li      a3,69
li      a4,69
li      a5,69
li      a6,69
li      a7,69
li      s2,69
li      s3,69
li      s4,69
li      s5,69
li      s6,0
li      s7,0
li      s8,0
li      s9,0
li      s10,0
li      s11,0
li      t3,0
li      t4,0
li      t5,0
li      t6,0
lui     sp,0x3
addi    sp,sp,2044 # 37fc <_bss_end+0x36c0>
sll     sp,sp,a0
.word 0x088000ef
li      a0,0
.word 0x04c000ef #<exit>
j       90 #<_start+0x90>
nop
nop
nop
nop
addi    sp,sp,-32
sw      s0,28(sp)
addi    s0,sp,32
sw      a0,-20(s0)
li      a5,276
lw      a5,0(a5)
lw      a4,-20(s0)
sw      a4,0(a5)
lw      a5,-20(s0)
mv      a0,a5
lw      s0,28(sp)
addi    sp,sp,32
ret
addi    sp,sp,-32
sw      s0,28(sp)
addi    s0,sp,32
sw      a0,-20(s0)
li      a5,280
lw      a5,0(a5)
lw      a4,-20(s0)
sw      a4,0(a5)
lw      a5,-20(s0)
mv      a0,a5
lw      s0,28(sp)
addi    sp,sp,32
ret
li      a0,111
ret