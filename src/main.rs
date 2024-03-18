#![no_std]
#![no_main]

use core::panic::PanicInfo;
use riscv_rt::entry;

fn wfi() {
    unsafe { core::arch::asm!("wfi") };
}

fn halt() -> ! {
    loop {
        wfi();
    }
}

#[entry]
fn main() -> ! {
    halt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt();
}
