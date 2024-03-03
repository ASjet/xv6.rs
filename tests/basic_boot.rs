#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(xv6::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use xv6::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    xv6::arch::halt();
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    xv6::test::panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
