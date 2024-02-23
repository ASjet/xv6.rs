#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod test;

mod vga;
mod asm;
mod serial;

const PANIC_INFO_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightRed, vga::Color::Black);

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    vga::set_color(PANIC_INFO_COLOR);
    println!("{}", info);
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("Hello, World!\ninteger: {}, float: {}", 42, 3.14);

    // panic!("Some panic message");

    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn trivial_assertion2() {
    assert_eq!(1, 1);
}
