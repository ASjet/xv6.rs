#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(xv6::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![allow(unused_imports)]

use bootloader::BootInfo;
use core::panic::PanicInfo;
use xv6::arch;
use xv6::dmesg;
use xv6::println;
use xv6::vga::{self, Color, ColorCode};

const PANIC_INFO_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    xv6::init();

    dmesg!(
        "physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );
    for &region in boot_info.memory_map.iter() {
        dmesg!(
            "start: {:#x}, end: {:#x}, type: {:?}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.region_type
        );
    }

    #[cfg(test)]
    test_main();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("Hello, World!\ninteger: {}, float: {}", 42, 3.14);

    // panic!("Some panic message");

    arch::halt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    xv6::with_color!(PANIC_INFO_COLOR, {
        println!("{}", info);
    });
    arch::halt();
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    xv6::test::panic_handler(info)
}
