#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga;
mod asm;

const PANIC_INFO_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightRed, vga::Color::Black);

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_color(PANIC_INFO_COLOR);
    println!("{}", info);
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("Hello, World!\ninteger: {}, float: {}", 42, 3.14);

    // panic!("Some panic message");

    #[cfg(test)]
    test_main();

    loop {}
}

const TEST_NORM_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::White, vga::Color::Black);
const TEST_INIT_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightCyan, vga::Color::Black);
const TEST_PASS_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightGreen, vga::Color::Black);
const TEST_FAIL_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightRed, vga::Color::Black);

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    vga::set_color(TEST_INIT_COLOR);
    println!("Running {} tests", tests.len());
    vga::set_color(TEST_NORM_COLOR);
    for test in tests {
        test();
    }

    asm::exit_qemu(asm::QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    vga::set_color(TEST_PASS_COLOR);
    println!("[ok]");
    vga::set_color(TEST_NORM_COLOR);
}
