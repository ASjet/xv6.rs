#![no_std] // don't link the Rust standard library
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::panic::PanicInfo;

pub mod arch;
pub mod serial;
pub mod test;
pub mod vga;

pub fn init() {
    arch::gdt::init();
    arch::interrupt::init_idt();
    arch::interrupt::init_pic();
}

// Entry point for unit test
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
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