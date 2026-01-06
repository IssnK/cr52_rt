/// UART register base address
pub const UART_BASE: usize = 0x9c090000;

/// UART Data Register
const UART_DR: *mut u32 = (UART_BASE + 0x00) as *mut u32;

/// UART Flag Register
const UART_FR: *mut u32 = (UART_BASE + 0x18) as *mut u32;

/// UART Line Control Register
const UART_LCRH: *mut u32 = (UART_BASE + 0x2c) as *mut u32;

/// UART Control Register
const UART_CR: *mut u32 = (UART_BASE + 0x30) as *mut u32;

/// UART Interrupt Mask Set/Clear Register
const UART_IMSC: *mut u32 = (UART_BASE + 0x38) as *mut u32;

/// UART Interrupt Clear Register
const UART_ICR: *mut u32 = (UART_BASE + 0x44) as *mut u32;

// Flag Register bits
const UART_FR_TXFF: u32 = 1 << 5;

// Line Control Register bits
const UART_LCRH_WLEN_8: u32 = 0b11 << 5;
const UART_LCRH_FEN: u32 = 1 << 4;

// Control Register bits
const UART_CR_UARTEN: u32 = 1 << 0;
const UART_CR_TXE: u32 = 1 << 8;
const UART_CR_RXE: u32 = 1 << 9;

// Interrupt Mask Register bits
const UART_IMSC_RXIM: u32 = 1 << 4;
const UART_IMSC_RTIM: u32 = 1 << 6;

/// UART peripheral structure
pub struct Uart;

impl Uart {
    /// Initialize the UART peripheral
    pub fn init() {
        unsafe {
            // Disable UART
            core::ptr::write_volatile(UART_CR, 0);

            // Clear all interrupts
            core::ptr::write_volatile(UART_ICR, 0x7FF);

            // Configure line control: 8-bit word length, enable FIFO
            core::ptr::write_volatile(UART_LCRH, UART_LCRH_WLEN_8 | UART_LCRH_FEN);

            // Enable UART, transmit and receive
            core::ptr::write_volatile(UART_CR, UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE);

            // Enable receive and receive timeout interrupts
            core::ptr::write_volatile(UART_IMSC, UART_IMSC_RXIM | UART_IMSC_RTIM);
        }
    }

    /// Write a single character to UART
    pub fn putc(c: u8) {
        unsafe {
            // Wait until the Transmit FIFO is not full
            while (core::ptr::read_volatile(UART_FR) & UART_FR_TXFF) != 0 {}

            // Write the character to the Data Register
            core::ptr::write_volatile(UART_DR, c as u32);
        }
    }

    /// Write a string to UART
    pub fn puts(s: &str) {
        for b in s.bytes() {
            Self::putc(b);
        }
    }

    /// Print a string to UART (alias for puts)
    #[inline]
    pub fn print(s: &str) {
        Self::puts(s);
    }
}

/// Initialize UART (convenience function)
#[inline]
pub fn uart_init() {
    Uart::init();
}

/// Write a single character to UART (convenience function)
#[inline]
pub fn uart_putc(c: u8) {
    Uart::putc(c);
}

/// Write a string to UART (convenience function)
#[inline]
pub fn uart_puts(s: &str) {
    Uart::puts(s);
}

/// Print a string to UART (convenience function)
#[inline]
pub fn print_uart(s: &str) {
    Uart::print(s);
}

/// Implement core::fmt::Write for UART to enable format! macros
impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::puts(s);
        Ok(())
    }
}
