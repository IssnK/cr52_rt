ENTRY(_reset)

MEMORY
{
    FLASH : ORIGIN = 0x88000000, LENGTH = 64M
    RAM   : ORIGIN = 0x00000000, LENGTH = 512M
}

SECTIONS
{
    .vector_table 0x00000000 : {
        KEEP(*(.text._vector_table));
    } > RAM

    /* コードセクション */
    .text : {
        KEEP(*(.boot .boot.*));
        *(.text .text.*);
        *(.rodata .rodata.*);
    } > RAM

    .data : AT(ADDR(.text) + SIZEOF(.text)) {
        _sdata = .;
        *(.data .data.*);
        _edata = .;
    } > RAM

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > RAM

    /* スタックセクション - ALIGNを修正 */
    .stack (NOLOAD) : ALIGN(8) {
        __stack_start__ = .;
        . += 0x1000;
        __stack = .;
        . += 0x400;
        __irq_stack = .;
        __stack_irq_top = .;
        . += 0x400;
        __fiq_stack = .;
        __stack_fiq_top = .;
        . += 0x400;
        __abort_stack = .;
        __stack_abt_top = .;
        . += 0x400;
        __undef_stack = .;
        . += 0x400;
        __supervisor_stack = .;
        __stack_svc_top = .;
    } > RAM

    /* ARM exception indexを破棄 */
    /DISCARD/ : {
        *(.ARM.exidx*)
        *(.ARM.extab*)
    }
}