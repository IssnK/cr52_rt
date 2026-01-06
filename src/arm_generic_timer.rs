use core::arch::asm;

pub struct ArmGenericTimer;

impl ArmGenericTimer {
    pub fn init() {
        unsafe {
            // The frequency should match Renode's config: 100 MHz
            asm!("mcr p15, 0, {}, c14, c0, 0", in(reg) 100_000_000u32);
            asm!("isb");
        }
    }

    /// Read the current physical timer count (CNTPCT_EL0)
    pub fn read_cntpct_el0() -> u64 {
        let low: u32;
        let high: u32;

        unsafe {
            asm!("mrrc p15, 0, {}, {}, c14", out(reg) low, out(reg) high);
        }

        ((high as u64) << 32) | (low as u64)
    }

    /// Read the frequency of the virtual timer (CNTFRQ_EL0)
    pub fn read_cntfrq_el0() -> u32 {
        let value: u32;
        unsafe {
            asm!("mrc p15, 0, {}, c14, c0, 0", out(reg) value);
        }
        value
    }

    pub fn enable_timer() {
        unsafe {
            asm!("mcr p15, 0, {}, c14, c2, 1", in(reg) 1u32); // Enable EL1 access to timers
            asm!("isb");
        }
    }

    pub fn disable_timer() {
        unsafe {
            asm!("mcr p15, 0, {}, c14, c2, 1", in(reg) 0u32); // Disable EL1 access to timers
        }
    }

    pub fn set_compare_value(value: u64) {
        let low = (value & 0xFFFFFFFF) as u32;
        let high = (value >> 32) as u32;

        unsafe {
            asm!("mcrr p15, 0, {}, {}, c14", in(reg) low, in(reg) high);
        }
    }

    pub fn set_control(enable: bool, imask: bool) {
        let mut control: u32 = 0;

        if enable {
            control |= 1 << 0; // ENABLE bit
        }
        if imask {
            control |= 1 << 1; // IMASK bit
        }
        // Bit 2 (ISTATUS) is read-only, don't try to set it

        unsafe {
            // Physical timer control register (CNTP_CTL)
            asm!("mcr p15, 0, {}, c14, c2, 1", in(reg) control);
            asm!("isb");
        }
    }

    // If you want to check the interrupt status:
    pub fn is_interrupt_pending() -> bool {
        let control: u32;
        unsafe {
            asm!("mrc p15, 0, {}, c14, c2, 1", out(reg) control);
        }
        (control & (1 << 2)) != 0 // Check ISTATUS bit
    }
}
