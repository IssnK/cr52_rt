/// GIC (Generic Interrupt Controller) Driver for ARM GICv3
///
/// This module provides initialization and control for the GICv3 interrupt controller,
/// with support for Software Generated Interrupts (SGIs).
use core::{
    arch::asm,
    ptr::{read_volatile, write_volatile},
};

// ==================== GIC Distributor (GICD) ====================
/// GIC Distributor base address
pub const GICD_BASE: usize = 0xAF000000;

/// GICD Control Register - Enable distributor
const GICD_CTLR: *mut u32 = (GICD_BASE + 0x0000) as *mut u32;

/// GICD Type Register - Provides information about the GIC configuration
const GICD_TYPER: *mut u32 = (GICD_BASE + 0x0004) as *mut u32;

/// GICD Interrupt Group Registers - Configure interrupt groups
const GICD_IGROUPR0: *mut u32 = (GICD_BASE + 0x0080) as *mut u32;

/// GICD Interrupt Set-Enable Registers - Enable interrupts
const GICD_ISENABLER0: *mut u32 = (GICD_BASE + 0x0100) as *mut u32;

/// GICD Interrupt Clear-Enable Registers - Disable interrupts
const GICD_ICENABLER0: *mut u32 = (GICD_BASE + 0x0180) as *mut u32;

/// GICD Interrupt Priority Registers - Set interrupt priorities (SGI 0-15)
const GICD_IPRIORITYR: *mut u32 = (GICD_BASE + 0x0400) as *mut u32;

/// GICD Software Generated Interrupt Register - Trigger SGIs
const GICD_SGIR: *mut u32 = (GICD_BASE + 0x0F00) as *mut u32;

// ==================== GIC Redistributor (GICR) ====================
/// GIC Redistributor base address
pub const GICR_BASE: usize = 0xAF100000;

/// GICR Control Register
const GICR_CTLR: *mut u32 = (GICR_BASE + 0x0000) as *mut u32;

/// GICR Waker Register - Controls power management
const GICR_WAKER: *mut u32 = (GICR_BASE + 0x0014) as *mut u32;

/// GICR Interrupt Group Register 0 (SGIs 0-31)
const GICR_IGROUPR0: *mut u32 = (GICR_BASE + 0x10000 + 0x0080) as *mut u32;

/// GICR Interrupt Set-Enable Register 0 (SGIs 0-31)
const GICR_ISENABLER0: *mut u32 = (GICR_BASE + 0x10000 + 0x0100) as *mut u32;

/// GICR Interrupt Clear-Enable Register 0 (SGIs 0-31)
const GICR_ICENABLER0: *mut u32 = (GICR_BASE + 0x10000 + 0x0180) as *mut u32;

/// GICR Interrupt Priority Registers (SGIs 0-31)
const GICR_IPRIORITYR: *mut u32 = (GICR_BASE + 0x10000 + 0x0400) as *mut u32;

// ==================== Constants ====================

/// Enable Group 0 interrupts
const GICD_CTLR_ENABLE_GRP0: u32 = 1 << 0;

/// Enable Group 1 interrupts
const GICD_CTLR_ENABLE_GRP1: u32 = 1 << 1;

/// Are We Awake bit in GICR_WAKER
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

/// SGI target list filter modes
const SGIR_TARGET_LIST_FILTER_SHIFT: u32 = 24;
const SGIR_TARGET_LIST_FILTER_MASK: u32 = 0x3 << SGIR_TARGET_LIST_FILTER_SHIFT;

/// SGI target list filter: Use target list
pub const SGI_TARGET_LIST: u32 = 0x0 << SGIR_TARGET_LIST_FILTER_SHIFT;

/// SGI target list filter: All except self
pub const SGI_TARGET_ALL_EXCEPT_SELF: u32 = 0x1 << SGIR_TARGET_LIST_FILTER_SHIFT;

/// SGI target list filter: Self only
pub const SGI_TARGET_SELF: u32 = 0x2 << SGIR_TARGET_LIST_FILTER_SHIFT;

/// Default priority for SGIs (higher number = lower priority)
const DEFAULT_SGI_PRIORITY: u8 = 0xA0;

// ==================== GIC Structure ====================

/// GIC (Generic Interrupt Controller) driver structure
pub struct Gic;

impl Gic {
    /// Initialize the GIC distributor and redistributor
    ///
    /// This function:
    /// 1. Disables the distributor
    /// 2. Configures interrupt groups
    /// 3. Wakes up the redistributor
    /// 4. Enables the distributor for both Group 0 and Group 1
    pub fn init() {
        unsafe {
            // ===== Initialize Distributor =====

            // Disable distributor while configuring
            write_volatile(GICD_CTLR, 0);

            // Configure SGIs (0-15) as Group 1 interrupts
            // Setting bit = Group 1, Clearing bit = Group 0
            let mut igroupr = read_volatile(GICD_IGROUPR0);
            igroupr |= 0xFFFF_FFFF; // SGIs 0-15 as Group 1
            write_volatile(GICD_IGROUPR0, igroupr);

            // Set default priority for SGIs (0-15)
            for i in 0..4 {
                // Each register holds 4 priorities (8 bits each)
                let priority_reg = GICD_IPRIORITYR.add(i);
                write_volatile(
                    priority_reg,
                    (DEFAULT_SGI_PRIORITY as u32) << 24
                        | (DEFAULT_SGI_PRIORITY as u32) << 16
                        | (DEFAULT_SGI_PRIORITY as u32) << 8
                        | (DEFAULT_SGI_PRIORITY as u32),
                );
            }

            // ===== Initialize Redistributor =====

            // Wake up the redistributor
            let mut waker = read_volatile(GICR_WAKER);
            waker &= !GICR_WAKER_PROCESSOR_SLEEP;
            write_volatile(GICR_WAKER, waker);

            // Wait for children to wake up
            while (read_volatile(GICR_WAKER) & GICR_WAKER_CHILDREN_ASLEEP) != 0 {
                // Spin wait
            }

            // Configure SGIs (0-15) as Group 1 in redistributor
            let mut gicr_igroupr = read_volatile(GICR_IGROUPR0);
            gicr_igroupr |= 0xFFFF_FFFF; // SGIs 0-15 as Group 1
            write_volatile(GICR_IGROUPR0, gicr_igroupr);

            // Enable all SGIs (0-15) in redistributor
            write_volatile(GICR_ISENABLER0, 0xFFFF_FFFF);

            // Set default priority for SGIs in redistributor
            for i in 0..4 {
                let priority_reg = GICR_IPRIORITYR.add(i);
                write_volatile(
                    priority_reg,
                    (DEFAULT_SGI_PRIORITY as u32) << 24
                        | (DEFAULT_SGI_PRIORITY as u32) << 16
                        | (DEFAULT_SGI_PRIORITY as u32) << 8
                        | (DEFAULT_SGI_PRIORITY as u32),
                );
            }

            // Enable timer interrupts in redistributor
            // (Assuming timer interrupt ID is 30 for Cortex-R52)
            let timer_interrupt_id: u8 = 30;
            let timer_mask = 1u32 << (timer_interrupt_id % 32);
            write_volatile(GICR_ISENABLER0, timer_mask);

            // Set timer interrupt as Group 1
            let mut gicr_igroupr_timer = read_volatile(GICR_IGROUPR0);
            gicr_igroupr_timer |= timer_mask;
            write_volatile(GICR_IGROUPR0, gicr_igroupr_timer);

            // Set default priority for timer interrupt
            let timer_priority_reg = GICR_IPRIORITYR.add((timer_interrupt_id / 4) as usize);
            let byte_offset = (timer_interrupt_id % 4) * 8;
            let mut current_priority = read_volatile(timer_priority_reg);
            let mask = 0xFF << byte_offset;
            current_priority =
                (current_priority & !mask) | ((DEFAULT_SGI_PRIORITY as u32) << byte_offset);
            write_volatile(timer_priority_reg, current_priority);

            Self::write_icc_ctlr(0); // Enable GIC CPU interface

            // ICC_PMR_EL1 - Set priority mask to allow all priorities
            asm!("mcr p15, 0, {}, c4, c6, 0", in(reg) 0xFFu32);
            asm!("isb");

            // Enable CPU interface for Group 1 interrupts
            let sre_value = 0b111; // Enable system register access, Group 0 and Group 1
            asm!("mcr p15, 0, {}, c12, c12, 7", in(reg) sre_value);
            asm!("isb");

            // Set Priority Mask to allow all interrupts (0xFF)
            // ICC_PMR: CP15, 0, Rt, c4, c6, 0
            asm!("mcr p15, 0, {}, c4, c6, 0", in(reg) 0xff);

            // Enable Group 1 interrupts in the CPU interface
            // ICC_IGRPEN1: CP15, 0, Rt, c12, c12, 7
            asm!("mcr p15, 0, {}, c12, c12, 7", in(reg) 1);

            // Inside Gic::init()
            // 1. Enable SRE
            asm!("mcr p15, 0, {}, c12, c12, 5", in(reg) 0x7u32);

            // 2. Set Priority Mask (PMR) to allow all
            asm!("mcr p15, 0, {}, c4, c6, 0", in(reg) 0xFFu32);

            // 3. Enable Group 1 Interrupts in CPU Interface
            asm!("mcr p15, 0, {}, c12, c12, 7", in(reg) 1u32);
            asm!("isb");

            // ===== Enable Distributor =====

            Self::write_icc_ctlr(0); // EOImode = 0

            // Enable distributor for both Group 0 and Group 1
            write_volatile(GICD_CTLR, GICD_CTLR_ENABLE_GRP0 | GICD_CTLR_ENABLE_GRP1);
        }
    }

    /// Read ICC_RPR (Running Priority Register)
    pub fn read_running_priority() -> u32 {
        let rpr: u32;
        unsafe {
            // ICC_RPR: CP15, 0, Rt, c12, c11, 3
            asm!(
                "mrc p15, 0, {}, c12, c11, 3",
                out(reg) rpr
            );
        }
        rpr
    }

    /// Read ICC_HPPIR1 (Highest Priority Pending Interrupt)
    pub fn read_highest_pending() -> u32 {
        let hppir: u32;
        unsafe {
            // ICC_HPPIR1: CP15, 0, Rt, c12, c12, 2
            asm!(
                "mrc p15, 0, {}, c12, c12, 2",
                out(reg) hppir
            );
        }
        hppir
    }

    /// Enable a specific SGI (Software Generated Interrupt)
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    pub fn enable_sgi(sgi_id: u8) {
        if sgi_id > 15 {
            return; // SGIs are only 0-15
        }

        unsafe {
            let mask = 1u32 << sgi_id;

            // Enable in GICR
            let current = read_volatile(GICR_ISENABLER0);
            write_volatile(GICR_ISENABLER0, current | mask);
        }
    }

    /// Read ICC_IAR1 (Interrupt Acknowledge Register - Group 1)
    /// Returns the full IAR value including source CPU info for SGIs
    /// For SGIs (0-15), bits [12:10] contain the source CPU ID
    pub fn read_interrupt_ack() -> u32 {
        let iar: u32;
        unsafe {
            asm!(
                "mrc p15, 0, {}, c12, c12, 0",
                out(reg) iar
            );
        }
        iar
    }

    /// Extract just the interrupt ID from an IAR value
    pub fn get_interrupt_id(iar: u32) -> u32 {
        iar & 0x3FF // Bits [9:0] contain the interrupt ID
    }

    /// Clear pending SGI by writing to GICR_ICPENDR0
    pub fn clear_sgi_pending(sgi_id: u8) {
        if sgi_id > 15 {
            return;
        }
        unsafe {
            let icpendr0 = (GICR_BASE + 0x10000 + 0x0280) as *mut u32;
            write_volatile(icpendr0, 1u32 << sgi_id);
        }
    }

    /// Write ICC_DIR (Deactivate Interrupt Register)
    /// Used when EOImode is set to 1 (separate priority drop and deactivation)
    pub fn write_deactivate_interrupt(interrupt_id: u32) {
        unsafe {
            // ICC_DIR: CP15, opc1=0, Rt, CRn=c12, CRm=c11, opc2=1
            asm!(
                "mcr p15, 0, {}, c12, c11, 1",
                in(reg) interrupt_id
            );
            asm!("isb");
        }
    }

    /// Write ICC_EOIR1 (End of Interrupt Register - Group 1)
    pub fn write_end_of_interrupt_group1(interrupt_id: u32) {
        unsafe {
            // ICC_EOIR1: CP15, opc1=0, Rt, CRn=c12, CRm=c12, opc2=1
            asm!(
                "mcr p15, 0, {}, c12, c12, 1",
                in(reg) interrupt_id
            );
            asm!("isb");
        }
    }

    /// Write ICC_EOIR0 (End of Interrupt Register - Group 0)
    pub fn write_end_of_interrupt_group0(interrupt_id: u32) {
        unsafe {
            // ICC_EOIR0: CP15, opc1=0, Rt, CRn=c12, CRm=c8, opc2=1
            asm!(
                "mcr p15, 0, {}, c12, c8, 1",
                in(reg) interrupt_id
            );
            asm!("isb");
        }
    }

    pub fn write_icc_ctlr(value: u32) {
        unsafe {
            asm!("mcr p15, 0, {}, c12, c12, 4", in(reg) value);
            asm!("isb");
        }
    }

    /// Disable a specific SGI
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    pub fn disable_sgi(sgi_id: u8) {
        if sgi_id > 15 {
            return; // SGIs are only 0-15
        }

        unsafe {
            let mask = 1u32 << sgi_id;

            // Disable in GICR
            write_volatile(GICR_ICENABLER0, mask);
        }
    }

    // Clear pending state of a specific SGI
    pub fn clear_pending_sgi(sgi_id: u8) {
        if sgi_id > 15 {
            return; // SGIs are only 0-15
        }
        unsafe {
            let mask = 1u32 << sgi_id;

            // Clear pending in GICR
            write_volatile(GICR_ICENABLER0, mask);
        }
    }

    /// Send a Software Generated Interrupt (SGI)
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    /// * `target_list` - Target CPU list (bit mask, bit 0 = CPU 0, etc.)
    /// * `filter` - Target filter mode (SGI_TARGET_LIST, SGI_TARGET_ALL_EXCEPT_SELF, or SGI_TARGET_SELF)
    ///
    /// # Examples
    /// ```
    /// // Send SGI 0 to CPU 1
    /// Gic::send_sgi(0, 0b10, SGI_TARGET_LIST);
    ///
    /// // Send SGI 1 to all CPUs except self
    /// Gic::send_sgi(1, 0, SGI_TARGET_ALL_EXCEPT_SELF);
    ///
    /// // Send SGI 2 to self
    /// Gic::send_sgi(2, 0, SGI_TARGET_SELF);
    /// ```

    pub fn send_sgi(sgi_id: u8, target_list: u16, filter: u32) {
        if sgi_id > 15 {
            return;
        }

        unsafe {
            // We map your 'filter' constants to the ICC_SGI1R IRM (Interrupt Routing Mode) bit
            // GICv3 IRM: 0 = Use Target List, 1 = All except self
            let irm = match filter {
                SGI_TARGET_ALL_EXCEPT_SELF => 1u32 << 24,
                _ => 0u32,
            };

            // ICC_SGI1R System Register Format (AArch32):
            // Rt (Lower 32 bits): [31:24] SGI ID, [23:16] TargetList, [24] IRM
            // Rt2 (Upper 32 bits): Affinity levels (0 for simple single-cluster setups)

            let val: u32 = irm | ((target_list as u32) << 0) | ((sgi_id as u32) << 24);

            // CP15, 0, Rt, Rt2, c12 (ICC_SGI1R)
            asm!(
                "mcrr p15, 0, {}, {}, c12",
                in(reg) val,
                in(reg) 0u32 // Affinity 0.0.0
            );
            asm!("isb");
        }
    }

    /// Send SGI to a specific CPU
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    /// * `cpu_id` - Target CPU ID (0-15)
    pub fn send_sgi_to_cpu(sgi_id: u8, cpu_id: u8) {
        if cpu_id > 15 {
            return; // Only support 16 CPUs max
        }

        let target_mask = 1u16 << cpu_id;
        Self::send_sgi(sgi_id, target_mask, SGI_TARGET_LIST);
    }

    /// Send SGI to all CPUs except the current one
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    pub fn send_sgi_to_all_except_self(sgi_id: u8) {
        Self::send_sgi(sgi_id, 0, SGI_TARGET_ALL_EXCEPT_SELF);
    }

    /// Send SGI to self
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)

    pub fn send_sgi_to_self(sgi_id: u8) {
        if sgi_id > 15 {
            return;
        }

        unsafe {
            // ICC_SGI1R (System Generated Interrupt Group 1 Register)
            // Format for AArch32 (Cortex-R52):
            // Rt = SGI ID and other controls, Rt2 = Affinity bits

            let sgi_id_u32 = sgi_id as u32;
            let irm = 1u32 << 24; // Interrupt Routing Mode: 1 = "All except self"
            // To send to SELF specifically, we use IRM = 0 and set the target list

            // Target list for CPU 0 in the current cluster is bit 0
            let target_list = 0b1u32;
            let val: u32 = (target_list << 0) | (sgi_id_u32 << 24);

            // ICC_SGI1R is a 64-bit register accessed via MCRR
            // CP15, 0, Rt, Rt2, c12
            asm!(
                "mcrr p15, 0, {}, {}, c12",
                in(reg) val,    // Lower 32 bits (Target List, etc)
                in(reg) 0u32    // Upper 32 bits (Affinity levels)
            );
            asm!("isb");
        }
    }

    /// Set the priority of an SGI
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    /// * `priority` - Priority value (0-255, lower value = higher priority)
    pub fn set_sgi_priority(sgi_id: u8, priority: u8) {
        if sgi_id > 15 {
            return;
        }

        unsafe {
            // Each priority register holds 4 priorities
            let reg_index = (sgi_id / 4) as usize;
            let byte_offset = (sgi_id % 4) * 8;

            let priority_reg = GICR_IPRIORITYR.add(reg_index);
            let mut current = read_volatile(priority_reg);

            // Clear the old priority and set new one
            let mask = 0xFF << byte_offset;
            current = (current & !mask) | ((priority as u32) << byte_offset);

            write_volatile(priority_reg, current);
        }
    }

    /// Read the GIC Distributor Type Register
    /// Returns information about the GIC configuration
    pub fn read_typer() -> u32 {
        unsafe { read_volatile(GICD_TYPER) }
    }

    /// Get the number of implemented interrupt lines
    pub fn get_num_interrupts() -> u32 {
        let typer = Self::read_typer();
        // ITLinesNumber field [4:0] indicates (N+1)*32 interrupts
        let it_lines = typer & 0x1F;
        (it_lines + 1) * 32
    }

    /// Check if a specific SGI is enabled
    ///
    /// # Arguments
    /// * `sgi_id` - SGI number (0-15)
    ///
    /// # Returns
    /// `true` if the SGI is enabled, `false` otherwise
    pub fn is_sgi_enabled(sgi_id: u8) -> bool {
        if sgi_id > 15 {
            return false;
        }

        unsafe {
            let enabled = read_volatile(GICR_ISENABLER0);
            (enabled & (1 << sgi_id)) != 0
        }
    }

    // Read ICC_SRE_EL1 to check if system register access is enabled
    /// Check if ICC_SRE_EL1 indicates system register access is enabled
    pub fn read_icc_sre() -> u32 {
        let value: u32;
        unsafe {
            asm!("mrc p15, 0, {}, c12, c12, 5", out(reg) value);
        }
        value
    }
}

// ==================== Convenience Functions ====================

/// Initialize the GIC (convenience function)
#[inline]
pub fn gic_init() {
    Gic::init();
}

/// Send an SGI to a specific CPU (convenience function)
#[inline]
pub fn send_sgi(sgi_id: u8, cpu_id: u8) {
    Gic::send_sgi_to_cpu(sgi_id, cpu_id);
}

/// Send an SGI to all CPUs except self (convenience function)
#[inline]
pub fn send_sgi_broadcast(sgi_id: u8) {
    Gic::send_sgi_to_all_except_self(sgi_id);
}

/// Send an SGI to self (convenience function)
#[inline]
pub fn send_sgi_to_self(sgi_id: u8) {
    Gic::send_sgi_to_self(sgi_id);
}
