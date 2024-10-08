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

### patching precompiled code vs making a real compiler

tradeoff: taking previously compiled RISC-V code and "bundling" it into the appropriate format for my VLIW or developing a proper compiler backend

Precompiled code:
- Can't schedule false dependencies away (need register renaming)
- Don't have lot of control if we want to implement new stuff in the hardware.
  - Say add some new architecturally visible registers for loop pipelining or something.
- less learning to be done - real compiler you implement register allocator, learn how to do the stack properly, etc.
- parsing binary RISC-V insns in rust kind of a pain (if coming directly from the elf)
- jump table not realistic
- Most likely it's going to be very specific to whatever compiler output I happen to test with it, so it can't just "translate any RISC-V binary"

### scrapped compilation pipeline idea

Start with ELF executable

* Convert to basic blocks, collect dependency information
  * grab dependency information, true dependencies work the same as from hw2
  * also put false dependencies: the overwriting of any register should depend on the last one to read or write it as well
* Fix auipc
  * Every auipc should have another instruction added which does an offset for where it should be. So if it's auipc at pc 0x180 on r0 then the next instruction should be addi, r0, r0, (0x180 - real address of auipc insn once scheduled)
  * TODO: assumption about where the data referenced is? presumably it is used to load from the .data section but if it's wrong then it's kind of screwed
  * GCC doesn't seem to use anything other than auipc for loading from data. In this case it should be fine, although we'd need to do analysis to figure out th

### notes about patching offsets

There seems to be a couple different addressing schemes we could be worried about 
- Branch, call offsets (obvious)
- Code for simple indirect jumps: the register which is used to do the offset is going to be shifted by a different value
  - Returns are OK as our processor would store the correct PC in the link register when calling.
- Code for jump tables: the addresses in the jump table are wrong (and the reference to the jump table itself is wrong)
- Any use of auipc: The PC is not what the rest of the code would have expected
- Any load of global data
  - Since we are not changing the ordering within the data section, it would be sufficient to simply add an offset corresponding to (new text size - old size)
  - Or just move the data to come first in the executable.
  - This could be loaded a number of ways:
    - Setting up a register to the address directly
    - auipc
    - Offset to x0 register

### auipc plan is foiled (kinda)

matmul contains some weird code generation using alternating auipc + store instruction
using it to access the data section, as before, but it's not generating the extra addi, so it's currently failing
in general, gcc is being very stupid here, -O2 is on but maybe clang would work better idk

lets use a loop instead of unrolling the entire matrix setup in matmul.c


### plan for now

Make a simple stupid "compiler" which just takes the output from GCC (parsed output from objdump, so I don't have to bother with parsing machine code) and gives a .hex which can be inserted into the original binary, formatted properly for the VLIW.

Handle auipc by assuming GCC will probably generate an offset addi insn wherever auipc shows up, and then just coalesce the two instructions together, that way we can adjust the PC offset as one.
- loop through instructions
- when auipc is encountered, store its pc
- the next instruction, we expect to be addi, we modify it with a label that stores the pc to the auipc that was just seen
- we schedule it as normal
- we remap the label by using the LUT just like all the other branch/jump instructions
- handle the Immediate enum type differently by adding the offset with this label (Immediate value - Old PC + new label)

Handle the size difference between the old and new program by adding some data in the first word of the program, storing the size difference between old and new. Every load except for MMIO has this offset applied. This way we don't have to do really any analysis to fix the addresses. (otherwise we'd have to have a way of knowing which instructions are manipulating MMIO and which aren't, and that doesn't sound easy, although it completely depends on what assumptions you make about the program)

Branches and jumps will be fixed before being generated. 
we will have some sort of hashmap that maps the addresses of the source instructions to the bundle that they ended up in.

## name brainstorm

VROOM ?

Very _ Out of Order Machine ?

Verifiably _ Out of Order Machine ?


## testing strategy 

Components:

- Elf -> Assembly
  - Manually verified parsed output was correct against objdump
- Scheduling/Optimization
  - Not currently tested
- Assembly -> Hex
  - Diff between objcopy'd hex from a binary and output with "-bv"
  - Test that the offsets are correct
  - Moved offsetting to before assembler stage to test
- CPU


## hardware ideas

in no order:

- Bitmask scoreboard instead of 8 ports
  - Compute a 32-bit mask of registers modified by each FU in the WB stage, OR them all together
  - Compute a 32-bit mask of registers modified by each FU in the decode stage, AND them all together
  - New scoreboard value is (current mask | WB mask) & decode mask.
  - 1 port 32-bit register instead of 8-port vector of 32-bit registers.
- NUCA
  - Fill FPGA without eating cost of routing.
- No scoreboarding except for load/store instructions.
  - All other instructions should have a fixed latency.
  - Bank memory instructions in their own register set

## GCC idiosyncrasies

- auipc generation:
  - it seems to default to auipc for global loads, and doesn't always add an offset with addi at the end 
  - for example in the case of a load/store it can be put directly into the offset for the instruction
- jump table:
  - as you would expect although it also does a jump to some address before doing jalr, it seems

