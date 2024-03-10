#![no_std] // don't link the Rust standard library
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod arch;
pub mod print;
pub mod test;
pub mod vga;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

pub fn init() {
    arch::init();
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for unit test
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    arch::halt();
}

// Panic handler for unit test
#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    test::panic_handler(info)
}
