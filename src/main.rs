#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

mod vga;

const PANIC_INFO_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::LightRed, vga::Color::Black);

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::WRITER.lock().set_color(PANIC_INFO_COLOR);
    println!("{}", info);
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("Hello, World!\ninteger: {}, float: {}", 42, 3.14);

    panic!("Some panic message");
}
