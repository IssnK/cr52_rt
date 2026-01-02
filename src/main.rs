#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const UART_BASE: usize = 0x9c090000;
const UART_DR: *mut u32 = (UART_BASE + 0x00) as *mut u32;
const UART_FR: *mut u32 = (UART_BASE + 0x18) as *mut u32;
const UART_LCRH: *mut u32 = (UART_BASE + 0x2c) as *mut u32;
const UART_CR: *mut u32 = (UART_BASE + 0x30) as *mut u32;
const UART_IMSC: *mut u32 = (UART_BASE + 0x38) as *mut u32;
const UART_ICR: *mut u32 = (UART_BASE + 0x44) as *mut u32;

const UART_FR_TXFF: u32 = 1 << 5;
const UART_LCRH_WLEN_8: u32 = 0b11 << 5;
const UART_LCRH_FEN: u32 = 1 << 4;
const UART_CR_UARTEN: u32 = 1 << 0;
const UART_CR_TXE: u32 = 1 << 8;
const UART_CR_RXE: u32 = 1 << 9;
const UART_IMSC_RXIM: u32 = 1 << 4;
const UART_IMSC_RTIM: u32 = 1 << 6;

unsafe fn write_icc_sre(value: u32) {
    asm!("mcr p15, 0, {}, c12, c12, 5", in(reg) value);
    asm!("isb");
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    uart_init();
    print_uart("System Booted in EL1\n");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_irq_handler() {
    print_uart("Interrupt Received!\n");
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_swi_handler() {
    print_uart("Software Interrupt (SWI) Called\n");
}

// Missing handler functions that your boot.s references
#[unsafe(no_mangle)]
pub extern "C" fn fiq_handler_asm() {
    print_uart("FIQ Handler Called\n");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_undef_handler() {
    print_uart("Undefined Instruction Exception\n");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_data_abort_handler() {
    print_uart("Data Abort Exception\n");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_prefetch_abort_handler() {
    print_uart("Prefetch Abort Exception\n");
    loop {}
}

fn uart_init() {
    unsafe {
        // Disable UART
        core::ptr::write_volatile(UART_CR, 0);
        // Clear all interrupts
        core::ptr::write_volatile(UART_ICR, 0x7FF);
        core::ptr::write_volatile(UART_LCRH, UART_LCRH_WLEN_8 | UART_LCRH_FEN);
        core::ptr::write_volatile(UART_CR, UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE);
        core::ptr::write_volatile(UART_IMSC, UART_IMSC_RXIM | UART_IMSC_RTIM);
    }
}

fn uart_putc(c: u8) {
    unsafe {
        // Wait until the Transmit FIFO is not full
        while (core::ptr::read_volatile(UART_FR) & UART_FR_TXFF) != 0 {}
        // Write the character to the Data Register
        core::ptr::write_volatile(UART_DR, c.into());
    }
}

fn uart_puts(s: &str) {
    for b in s.bytes() {
        uart_putc(b);
    }
}

fn print_uart(s: &str) {
    uart_puts(s);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_uart("PANIC!\n");
    loop {}
}
