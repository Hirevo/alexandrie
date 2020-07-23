#include "psm.h"

.text
.globl rust_psm_stack_direction
.p2align 2
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
/* extern "C" fn() -> u8 */
.cfi_startproc
    li x10, STACK_DIRECTION_DESCENDING
    jr x1
.rust_psm_stack_direction_end:
.size       rust_psm_stack_direction,.rust_psm_stack_direction_end-rust_psm_stack_direction
.cfi_endproc


.globl rust_psm_stack_pointer
.p2align 2
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
/* extern "C" fn() -> *mut u8 */
.cfi_startproc
    add x10, x2, x0
    jr x1
.rust_psm_stack_pointer_end:
.size       rust_psm_stack_pointer,.rust_psm_stack_pointer_end-rust_psm_stack_pointer
.cfi_endproc


.globl rust_psm_replace_stack
.p2align 2
.type rust_psm_replace_stack,@function
rust_psm_replace_stack:
/* extern "C" fn(x10: usize, x11: extern "C" fn(usize), x12: *mut u8) */
.cfi_startproc
    add x2, x12, x0
    jr x11
.rust_psm_replace_stack_end:
.size       rust_psm_replace_stack,.rust_psm_replace_stack_end-rust_psm_replace_stack
.cfi_endproc


.globl rust_psm_on_stack
.p2align 2
.type rust_psm_on_stack,@function
rust_psm_on_stack:
/* extern "C" fn(x10: usize, x11: usize, x12: extern "C" fn(usize, usize), x13: *mut u8) */
.cfi_startproc
    sd x1, -8(x13)
    sd x2, -16(x13)
    .cfi_def_cfa x13, 0
    .cfi_offset x1, -8
    .cfi_offset x2, -16
    addi x2, x13, -16
    .cfi_def_cfa x2, -16
    jalr x1, x12, 0
    ld x1, 8(x2)
    .cfi_restore x1
    ld x2, 0(x2)
    .cfi_restore x2
    jr x1
.rust_psm_on_stack_end:
.size       rust_psm_on_stack,.rust_psm_on_stack_end-rust_psm_on_stack
.cfi_endproc
