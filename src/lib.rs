#![no_std]

pub mod arm_generic_timer;
pub mod gic;
pub mod handlers;
pub mod irq;
pub mod system;
pub mod uart;

// Re-export commonly used items
pub use arm_generic_timer::*;
pub use gic::*;
pub use handlers::*;
pub use irq::*;
pub use system::*;
pub use uart::*;

#[cfg(feature = "panic-handler")]
use core::panic::PanicInfo;

#[cfg(feature = "panic-handler")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::print_uart("PANIC!\n");
    loop {}
}
