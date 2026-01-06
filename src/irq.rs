use core::arch::asm;

pub fn enable_irq() {
    unsafe {
        asm!("cpsie i");
    }
}

pub fn disable_irq() {
    unsafe {
        asm!("cpsid i");
    }
}

pub fn wait_for_interrupt() {
    unsafe {
        asm!("wfi");
    }
}

pub fn enable_fiq() {
    unsafe {
        asm!("cpsie f");
    }
}
pub fn disable_fiq() {
    unsafe {
        asm!("cpsid f");
    }
}
