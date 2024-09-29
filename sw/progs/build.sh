#!/bin/bash
ELF2HEX=../../tools/elf2hex
riscv64-unknown-elf-gcc -march=rv32i -mabi=ilp32 -fno-builtin -static -nostdlib -nostartfiles -mcmodel=medany -o build/test32 -Tmmio.ld test.S
$ELF2HEX/elf2hex build/test32 0 16G build/test32.hex