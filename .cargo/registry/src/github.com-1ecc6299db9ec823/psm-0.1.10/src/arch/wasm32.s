#include "psm.h"

.text
.section .text.rust_psm_stack_direction,"",@
.globl rust_psm_stack_direction
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
.functype rust_psm_stack_direction () -> (i32)
    i32.const $STACK_DIRECTION_DESCENDING
    end_function
.rust_psm_stack_direction_end:
.size rust_psm_stack_direction, .rust_psm_stack_direction_end-rust_psm_stack_direction


.section .text.rust_psm_stack_pointer,"",@
.globl rust_psm_stack_pointer
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
.functype rust_psm_stack_pointer () -> (i32)
    global.get __stack_pointer
    end_function
.rust_psm_stack_pointer_end:
.size rust_psm_stack_pointer, .rust_psm_stack_pointer_end-rust_psm_stack_pointer

.globaltype __stack_pointer, i32
