#include "psm.h"
/* NOTE: sysv64 calling convention is used on all x86_64 targets, including Windows! */

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
.p2align 4
TYPE(rust_psm_stack_direction)
FUNCTION(rust_psm_stack_direction):
/* extern "sysv64" fn() -> u8 (%al) */
.cfi_startproc
    movb $STACK_DIRECTION_DESCENDING, %al # always descending on x86_64
    retq
.rust_psm_stack_direction_end:
SIZE(rust_psm_stack_direction,.rust_psm_stack_direction_end)
.cfi_endproc


GLOBL(rust_psm_stack_pointer)
.p2align 4
TYPE(rust_psm_stack_pointer)
FUNCTION(rust_psm_stack_pointer):
/* extern "sysv64" fn() -> *mut u8 (%rax) */
.cfi_startproc
    leaq 8(%rsp), %rax
    retq
.rust_psm_stack_pointer_end:
SIZE(rust_psm_stack_pointer,.rust_psm_stack_pointer_end)
.cfi_endproc


GLOBL(rust_psm_replace_stack)
.p2align 4
TYPE(rust_psm_replace_stack)
FUNCTION(rust_psm_replace_stack):
/* extern "sysv64" fn(%rdi: usize, %rsi: extern "sysv64" fn(usize), %rdx: *mut u8) */
.cfi_startproc
/*
    All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi.

    8-byte offset necessary to account for the "return" pointer that would otherwise be placed onto
    stack with a regular call
*/
    leaq -8(%rdx), %rsp
    jmpq *%rsi
.rust_psm_replace_stack_end:
SIZE(rust_psm_replace_stack,.rust_psm_replace_stack_end)
.cfi_endproc


GLOBL(rust_psm_on_stack)
.p2align 4
TYPE(rust_psm_on_stack)
FUNCTION(rust_psm_on_stack):
/* extern "sysv64" fn(%rdi: usize, %rsi: usize, %rdx: extern "sysv64" fn(usize, usize), %rcx: *mut u8) */
.cfi_startproc
    pushq %rbp
    .cfi_def_cfa %rsp, 16
    .cfi_offset %rbp, -16
    movq  %rsp, %rbp
    .cfi_def_cfa_register %rbp
    movq  %rcx, %rsp
    callq *%rdx
    movq  %rbp, %rsp
    popq  %rbp
    .cfi_def_cfa %rsp, 8
    retq
.rust_psm_on_stack_end:
SIZE(rust_psm_on_stack,.rust_psm_on_stack_end)
.cfi_endproc
