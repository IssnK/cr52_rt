    .syntax unified
    .arch armv8-r
    .arm

/* boot.s */
.section .text._vector_table, "ax", %progbits
.align 5
.global _vector_table

_vector_table:
    b _reset                /* 0x00: Reset */
    b undef_handler_asm     /* 0x04: Undefined Instruction */
    b swi_handler_asm       /* 0x08: Software Interrupt (SVC) */
    b prefetch_abort_asm    /* 0x0C: Prefetch Abort */
    b data_abort_asm        /* 0x10: Data Abort */
    b .                     /* 0x14: Reserved */
    b irq_handler_asm       /* 0x18: IRQ */
    b fiq_handler_asm       /* 0x1C: FIQ */

/* CPU Mode definitions */
.equ ARM_MODE_USR, 0x10
.equ ARM_MODE_FIQ, 0x11
.equ ARM_MODE_IRQ, 0x12
.equ ARM_MODE_SVC, 0x13
.equ ARM_MODE_ABT, 0x17
.equ ARM_MODE_UND, 0x1B
.equ ARM_MODE_SYS, 0x1F
.equ ARM_MODE_HYP, 0x1A

/* CPSR bit definitions */
.equ I_BIT, 0x80    /* IRQ disable bit */
.equ F_BIT, 0x40    /* FIQ disable bit */

.section .boot, "ax", %progbits
.align 4
.global _reset

_reset:
    cpsid if                     @ Disable IRQ and FIQ

    /* 1. Initialize EL2 (Hypervisor) */
    /* Set the Hyp Vector Base Address */
    ldr r0, =_vector_table
    mcr p15, 4, r0, c12, c0, 0   @ HVBAR
    dsb sy
    isb

    mrs r0, cpsr
    bic r0, r0, #0x1F             @ Clear mode bits
    orr r0, r0, #0x13             @ Set to Hyp mode (0b10011)
    msr spsr_cxsf, r0

    /* Set ELR_hyp to our EL1 entry point */
    ldr r0, =el1_entry
    msr elr_hyp, r0

    /* Enable Timer access from EL1 */
    mov r0, #0x1                  @ Enable EL1 access to physical timer
    mcr p15, 4, r0, c14, c2, 0
    isb

    /* 4. Transition to EL1 */
    eret

el1_entry:
    /* Now in EL1 (SVC mode) */
    
    /* 6. Setup Stacks */
    /* IRQ Mode */
    mrs r0, cpsr
    bic r0, r0, #0x1F             @ Clear mode bits
    orr r0, r0, #(ARM_MODE_IRQ | I_BIT | F_BIT)  @ IRQ Mode with IRQ/FIQ disabled
    msr cpsr_c, r0
    ldr sp, =__stack_irq_top

    /* FIQ Mode */
    mrs r0, cpsr
    bic r0, r0, #0x1F             @ Clear mode bits
    orr r0, r0, #(ARM_MODE_FIQ | I_BIT | F_BIT)  @ FIQ Mode with IRQ/FIQ disabled
    msr cpsr_c, r0
    ldr sp, =__stack_fiq_top

    /* Abort Mode */
    mrs r0, cpsr
    bic r0, r0, #0x1F             @ Clear mode bits
    orr r0, r0, #(ARM_MODE_ABT | I_BIT | F_BIT)  @ Abort Mode with IRQ/FIQ disabled
    msr cpsr_c, r0
    ldr sp, =__stack_abt_top

    /* SVC Mode */
    mrs r0, cpsr
    bic r0, r0, #0x1F             @ Clear mode bits
    orr r0, r0, #(ARM_MODE_SVC | I_BIT | F_BIT)  @ SVC Mode with IRQ/FIQ disabled
    msr cpsr_c, r0
    ldr sp, =__stack_svc_top

    /* 7. Clear BSS (Recommended to do in ASM for Rust) */
    ldr r0, =__bss_start
    ldr r1, =__bss_end
    mov r2, #0

1:
    cmp r0, r1
    bge 2f
    str r2, [r0], #4
    b 1b
2:
    dsb sy
    isb

    /* Jump to Rust */
    bl rust_main

halt_loop:
    b halt_loop

/* --- Exception Handlers --- */

.align 4
undef_handler_asm:
    push {r0-r3, r12, lr}
    bl rust_undef_handler
    pop {r0-r3, r12, lr}
    movs pc, lr

data_abort_asm:
    push {r0-r3, r12, lr}
    bl rust_data_abort_handler
    pop {r0-r3, r12, lr}
    movs pc, lr

prefetch_abort_asm:
    push {r0-r3, r12, lr}
    bl rust_prefetch_abort_handler
    pop {r0-r3, r12, lr}
    movs pc, lr

swi_handler_asm:
    push {r0-r3, r12, lr}
    mov r0, sp                   @ Pass stack pointer to Rust as context
    bl rust_swi_handler
    pop {r0-r3, r12, lr}
    movs pc, lr

irq_handler_asm:
    sub lr, lr, #4               @ Correct LR for IRQ return
    push {r0-r3, r12, lr}        @ Push context
    bl rust_irq_handler          @ Your Rust IRQ dispatcher
    pop {r0-r3, r12, lr}         @ Restore
    movs pc, lr                  @ Return to interrupted code

halt:
    wfe
    b halt
