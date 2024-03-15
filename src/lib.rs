#![no_std] // don't link the Rust standard library
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod arch;
pub mod mem;
pub mod print;
pub mod test;
pub mod vga;

extern crate alloc;

use bootloader::BootInfo;
use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::entry_point;

pub fn init(boot_info: &'static BootInfo) {
    arch::init(boot_info);
    mem::alloc::init(arch::vm::HEAP_START, arch::vm::HEAP_SIZE);
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for unit test
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    arch::halt();
}

// Panic handler for unit test
#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    test::panic_handler(info)
}
