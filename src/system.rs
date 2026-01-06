use core::arch::asm;

/// Write to the ICC_SRE register (System Register Enable)
/// This is used to enable access to GIC CPU interface system registers
pub unsafe fn write_icc_sre(value: u32) {
    asm!("mcr p15, 0, {}, c12, c12, 5", in(reg) value);
    asm!("isb");
}

/// Read from the ICC_SRE register
pub unsafe fn read_icc_sre() -> u32 {
    let value: u32;
    asm!("mrc p15, 0, {}, c12, c12, 5", out(reg) value);
    value
}
