#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(xv6::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::panic::PanicInfo;

use xv6::asm;
use xv6::println;
use xv6::vga::{self, Color, ColorCode};

const PANIC_INFO_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);

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

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_color(PANIC_INFO_COLOR);
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    xv6::test::panic_handler(info)
}
