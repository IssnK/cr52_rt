/* boot.s */
.section .text._vector_table, "ax", %progbits
.align 6   /* ARMv8-R requires 64-byte alignment for VBAR */
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

.section .boot, "ax", %progbits
.global _reset

_reset:
    /* 1. Initialize EL2 (Hypervisor) */
    /* Set the Hyp Vector Base Address */
    ldr r0, =_vector_table
    mcr p15, 4, r0, c12, c0, 0   @ HVBAR

    /* 2. Configure HCR (Hypervisor Configuration Register) 
       Set bit 31 (Register Width) to 0 for AArch32 EL1 */
    mov r0, #0
    mcr p15, 4, r0, c1, c1, 0

    /* 3. Prepare to drop to EL1 (SVC Mode) */
    /* SPSR_hyp: M[4:0] = 0b10011 (SVC), F/I/A bits masked (0x1D3) */
    ldr r0, =0x1D3
    msr spsr_hyp, r0

    /* Set ELR_hyp to our EL1 entry point */
    ldr r0, =el1_entry
    msr elr_hyp, r0

    /* 4. Transition to EL1 */
    eret

el1_entry:
    /* Now in EL1 (SVC mode) */
    /* Set EL1 Vector Base Address */
    ldr r0, =_vector_table
    mcr p15, 0, r0, c12, c0, 0   @ VBAR

    /* 5. Enable FPU/SIMD (Required for Rust) */
    mrc p15, 0, r0, c1, c0, 2    @ CPACR
    orr r0, r0, #(0xF << 20)     @ Enable CP10 and CP11
    mcr p15, 0, r0, c1, c0, 2
    isb
    mov r0, #0x40000000          @ FPEXC Enable bit
    vmsr fpexc, r0

    /* 6. Setup Stacks */
    cps #0x12                    @ IRQ Mode
    ldr sp, =__stack_irq_top
    
    cps #0x11                    @ FIQ Mode
    ldr sp, =__stack_fiq_top

    cps #0x17                    @ Abort Mode
    ldr sp, =__stack_abt_top

    cps #0x13                    @ SVC Mode (Return to default)
    ldr sp, =__stack_svc_top

    /* 7. Clear BSS (Recommended to do in ASM for Rust) */
    ldr r0, =__bss_start
    ldr r1, =__bss_end
    mov r2, #0

    /* Jump to Rust */
    bl rust_main

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
