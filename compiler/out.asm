BasicBlock 0:
0 | mv x12, x10 | li x15, 0 |                    |                    |
1 | li x10, 0 | li x16, 32 |                    |                    |
BasicBlock 1:
2 | sra x14, x11, x15 | sll x13, x12, x15 |                    |                    |
3 | andi x14, x14, 1 | addi x15, x15, 1 |                    | beq x14, x0, 4 |
BasicBlock 2:
4 | add x10, x10, x13 |                    |                    | bne x15, x16, 2 |
BasicBlock 3:
5 |                    |                    |                    | ret |
BasicBlock 4:
6 | li x1, 0 | li x2, 0 |                    |                    |
7 | li x3, 0 | li x4, 0 |                    |                    |
8 | li x5, 0 | li x6, 0 |                    |                    |
9 | li x7, 0 | li x8, 0 |                    |                    |
10 | li x9, 0 | li x11, 0 |                    |                    |
11 | li x12, 0 | li x13, 0 |                    |                    |
12 | li x14, 0 | li x15, 0 |                    |                    |
13 | li x16, 0 | li x17, 0 |                    |                    |
14 | li x18, 0 | li x19, 0 |                    |                    |
15 | li x20, 0 | li x21, 0 |                    |                    |
16 | li x22, 0 | li x23, 0 |                    |                    |
17 | li x24, 0 | li x25, 0 |                    |                    |
18 | li x26, 0 | li x27, 0 |                    |                    |
19 | li x28, 0 | li x29, 0 |                    |                    |
20 | li x30, 0 | li x31, 0 |                    |                    |
21 | lui x2, 3 |                    |                    |                    |
22 | addi x2, x2, 2044 |                    |                    |                    |
23 | sll x2, x2, x10 |                    |                    | jal 48 |
BasicBlock 5:
24 | li x10, 0 |                    |                    | jal 37 |
BasicBlock 6:
25 |                    |                    |                    | j 25 |
BasicBlock 7:
26 | addi x2, x2, -32 | li x15, 496 |                    |                    |
27 |                    |                    | sw x1, 28(x2) |                    |
28 |                    |                    | sw x8, 24(x2) |                    |
29 | addi x8, x2, 32 |                    | lw x15, 0(x15) |                    |
30 |                    |                    | sw x10, -20(x8) |                    |
31 |                    |                    | lw x14, -20(x8) |                    |
32 |                    |                    | sw x14, 0(x15) |                    |
33 |                    |                    | lw x15, -20(x8) |                    |
34 | mv x10, x15 |                    | lw x1, 28(x2) |                    |
35 |                    |                    | lw x8, 24(x2) |                    |
36 | addi x2, x2, 32 |                    |                    | ret |
BasicBlock 8:
37 | addi x2, x2, -32 | li x15, 500 |                    |                    |
38 |                    |                    | sw x1, 28(x2) |                    |
39 |                    |                    | sw x8, 24(x2) |                    |
40 | addi x8, x2, 32 |                    | lw x15, 0(x15) |                    |
41 |                    |                    | sw x10, -20(x8) |                    |
42 |                    |                    | lw x14, -20(x8) |                    |
43 |                    |                    | sw x14, 0(x15) |                    |
44 |                    |                    | lw x15, -20(x8) |                    |
45 | mv x10, x15 |                    | lw x1, 28(x2) |                    |
46 |                    |                    | lw x8, 24(x2) |                    |
47 | addi x2, x2, 32 |                    |                    | ret |
BasicBlock 9:
48 | addi x2, x2, -16 | li x12, 0 |                    |                    |
49 | li x15, 0 | li x16, 3 | sw x1, 12(x2) |                    |
50 | li x10, 2 | li x11, 32 | sw x8, 8(x2) |                    |
51 | sra x14, x16, x15 | sll x13, x10, x15 | sw x9, 4(x2) |                    |
52 | andi x14, x14, 1 | addi x15, x15, 1 |                    | beq x14, x0, 53 |
BasicBlock 10:
53 | add x12, x12, x13 |                    |                    | bne x15, x11, 51 |
BasicBlock 11:
54 | li x15, 6 | auipc x8, 0 |                    |                    |
55 | addi x8, x8, 96 | auipc x9, 0 |                    |                    |
56 | addi x9, x9, 102 |                    |                    | beq x12, x15, 65 |
BasicBlock 12:
57 |                    |                    | lbu x10, 0(x8) |                    |
58 | addi x8, x8, 1 |                    |                    | jal 26 |
BasicBlock 13:
59 |                    |                    |                    | bne x8, x9, 57 |
BasicBlock 14:
60 | li x10, 1 |                    |                    | jal 37 |
BasicBlock 15:
61 | li x10, 0 |                    | lw x1, 12(x2) |                    |
62 |                    |                    | lw x8, 8(x2) |                    |
63 |                    |                    | lw x9, 4(x2) |                    |
64 | addi x2, x2, 16 |                    |                    | ret |
BasicBlock 16:
65 | li x10, 79 |                    |                    | jal 26 |
BasicBlock 17:
66 | li x10, 107 |                    |                    | jal 26 |
BasicBlock 18:
67 | li x10, 0 |                    |                    | jal 37 |
BasicBlock 19:
68 |                    |                    |                    | j 61 |

