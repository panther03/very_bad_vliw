#include "encoding.h"

# define LREG lw
# define SREG sw
# define REGBYTES 4

#define stack_start 0x3800 - 4

.section ".text.init"
#  .text
#  .align 6
  .globl _start
_start:
  li  x1, 0
  li  x2, 0
  li  x3, 0
  li  x4, 0
  li  x5, 0
  li  x6, 0
  li  x7, 0
  li  x8, 0
  li  x9, 0
  li  x10,0
  li  x11,0
  li  x12,0
  li  x13,0
  li  x14,0
  li  x15,0
  li  x16,0
  li  x17,0
  li  x18,0
  li  x19,0
  li  x20,0
  li  x21,0
  li  x22,0
  li  x23,0
  li  x24,0
  li  x25,0
  li  x26,0
  li  x27,0
  li  x28,0
  li  x29,0
  li  x30,0
  li  x31,0

  li sp, stack_start
  sll sp, sp, x10

  call main

  li x10, 0

  call exit
1:
  j 1b

#.section ".tdata.begin"
#.globl _tdata_begin
#_tdata_begin:
#
#.section ".tdata.end"
#.globl _tdata_end
#_tdata_end:
#
#.section ".tbss.end"
#.globl _tbss_end
#_tbss_end:
