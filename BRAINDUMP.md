## Progress

TODO: why is _start not at 0 in the elf output ?

- [x] Parsing of objdump output 
- [x] Coalesce pass for auipc
- [x] Split into basic blocks
- [X] Updated dependency analysis (w/ False dependencies)
- [X] Update scheduling
- [X] Update labels of branch/jump
- [X] Patching hex file and adding offset for loads
- [X] Assemble output into hex or figure out why GCC is still generating jump + branch for one branch
- [X] CPU support for global load offset
- [X] mul32 runs
- [ ] Compile & run matmul32 + Show that IPC is > 1
- [X] Test a function pointer, see what code is generated

## Random Notes

## name brainstorm

VROOM ?

Very _ Out of Order Machine ?

Verifiably _ Out of Order Machine ?

Chat: 

VILAW – Very Incompetent Little Algorithm Wizard
VLISP – Very Large Instructions, Small Processor 
VLAIW – Virtually Lost Amidst Instruction Woes
VLIPS – Very Low Intelligence Processing System

prompt:
>> "Hey, I'm building a VLIW CPU and I want to give it a call of some sort that will be a silly funny acronym involving VLIW or something like that." Next prompt: "What if we would add one more letter somewhere? So that it would be a funny and silly acronym, that would also sound nice when read just the letters?"

## GCC idiosyncrasies

- auipc generation:
  - it seems to default to auipc for global loads, and doesn't always add an offset with addi at the end 
  - for example in the case of a load/store it can be put directly into the offset for the instruction
- jump table:
  - as you would expect although it also does a jump to some address before doing jalr, it seems

