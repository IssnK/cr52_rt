use crate::uart::print_uart;

// Default handler implementations are only compiled if the "default-handlers" feature is enabled
// This allows applications to provide their own handlers without conflicts

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_irq_handler() {
    print_uart("Interrupt Received!\n");
}

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_swi_handler() {
    print_uart("Software Interrupt (SWI) Called\n");
}

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn fiq_handler_asm() {
    print_uart("FIQ Handler Called\n");
    loop {}
}

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_undef_handler() {
    print_uart("Undefined Instruction Exception\n");
    loop {}
}

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_data_abort_handler() {
    print_uart("Data Abort Exception\n");
    loop {}
}

#[cfg(feature = "default-handlers")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_prefetch_abort_handler() {
    print_uart("Prefetch Abort Exception\n");
    loop {}
}
