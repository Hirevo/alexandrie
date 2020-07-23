#include "psm.h"

.text

#if CFG_TARGET_OS_darwin || CFG_TARGET_OS_macos || CFG_TARGET_OS_ios

#define GLOBL(fnname) .globl _##fnname
#define TYPE(fnname)
#define FUNCTION(fnname) _##fnname
#define SIZE(fnname,endlabel)

#else

#define GLOBL(fnname) .globl fnname
#define TYPE(fnname) .type fnname,@function
#define FUNCTION(fnname) fnname
#define SIZE(fnname,endlabel) .size fnname,endlabel-fnname

#endif


GLOBL(rust_psm_stack_direction)
.p2align 2
TYPE(rust_psm_stack_direction)
FUNCTION(rust_psm_stack_direction):
/* extern "C" fn() -> u8 */
.cfi_startproc
    orr w0, wzr, #STACK_DIRECTION_DESCENDING
    ret
.rust_psm_stack_direction_end:
SIZE(rust_psm_stack_direction,.rust_psm_stack_direction_end)
.cfi_endproc


GLOBL(rust_psm_stack_pointer)
.p2align 2
TYPE(rust_psm_stack_pointer)
FUNCTION(rust_psm_stack_pointer):
/* extern "C" fn() -> *mut u8 */
.cfi_startproc
    mov x0, sp
    ret
.rust_psm_stack_pointer_end:
SIZE(rust_psm_stack_pointer,.rust_psm_stack_pointer_end)
.cfi_endproc


GLOBL(rust_psm_replace_stack)
.p2align 2
TYPE(rust_psm_replace_stack)
FUNCTION(rust_psm_replace_stack):
/* extern "C" fn(r0: usize, r1: extern "C" fn(usize), r2: *mut u8) */
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
    mov sp, x2
    br x1
.rust_psm_replace_stack_end:
SIZE(rust_psm_replace_stack,.rust_psm_replace_stack_end)
.cfi_endproc


GLOBL(rust_psm_on_stack)
.p2align 2
TYPE(rust_psm_on_stack)
FUNCTION(rust_psm_on_stack):
/* extern "C" fn(r0: usize, r1: usize, r2: extern "C" fn(usize, usize), r3: *mut u8) */
.cfi_startproc
    stp x29, x30, [sp, #-16]!
    .cfi_def_cfa sp, 16
    mov x29, sp
    .cfi_def_cfa x29, 16
    .cfi_offset x29, -16
    .cfi_offset x30, -8
    mov sp, x3
    blr x2
    mov sp, x29
    .cfi_def_cfa sp, 16
    ldp x29, x30, [sp], #16
    .cfi_def_cfa sp, 0
    .cfi_restore x29
    .cfi_restore x30
    ret
.rust_psm_on_stack_end:
SIZE(rust_psm_on_stack,.rust_psm_on_stack_end)
.cfi_endproc
