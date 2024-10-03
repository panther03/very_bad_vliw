.word 0x340000ef
addi	t6,t6,-1
sb	t6,-1(t6)
addi	t6,t6,-1
lui	t6,0xffffe
lui	t6,0x1
jal	t6,0x55d68 
beq t6,t6,0b101010101010
