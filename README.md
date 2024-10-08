# A very bad VLIW CPU implementation

Including a very bad VLIW compiler

### Progress

- first VLIW implementation
- compile/transpile CS629 tests to VLIW

### Goals

- improve circuit to get >1 IPC on CS629 tests
- super basic 8086 interpreter
- Instruction compression (v2 VLIW)
- Real compiler from LLVM or QBE
- 8086 jit (no elp from HW; not using dynamic traces to compile)
  - study how existing ones work
- fancy DBT
- I/O 
- boot DOS
- FPGA