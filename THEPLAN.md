
Compilation pipeline:

Start with ELF executable

* Convert to basic blocks, collect dependency information
  * grab dependency information, true dependencies work the same as from hw2
  * also put false dependencies: the overwriting of any register should depend on the last one to read or write it as well
* Fix auipc
  * Every auipc should have another instruction added which does an offset for where it should be. So if it's auipc at pc 0x180 on r0 then the next instruction should be addi, r0, r0, (0x180 - real address of auipc insn once scheduled)
  * TODO: assumption about where the data referenced is? presumably it is used to load from the .data section but if it's wrong then it's kind of screwed
  * GCC doesn't seem to use anything other than auipc for loading from data. In this case it should be fine, although we'd need to do analysis to figure out th

## Offsets 

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

This thing kind of sucks anyway though because it won't be able to schedule false dependencies away.
And we don't have lot of control if we want to implement new stuff in the hardware.
And there's less learning to be done.
And I don't want to parse RISC-V instructions in Rust, no thanks.
And most likely it's going to be very specific to whatever compiler output I happen to test with it, so it can't just "translate any RISC-V binary"
 

## random notes

- python driver.py script to process the elf file and patch it to be a VLIW hex file
- parsing of assembly instructions as before 
- coalescing of auipc with proceding instruction, panic if fail
- writing out of jal as .word 
- 