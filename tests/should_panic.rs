#![no_std]
#![no_main]

use core::panic::PanicInfo;
use xv6::asm;
use xv6::{serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("should_panic::should_panic...\t");
    should_panic();
    serial_println!("[test did not panic]");
    asm::exit_qemu(asm::QemuExitCode::Failure);
    loop {}
}

#[panic_handler]
pub fn panic(_: &PanicInfo) -> ! {
    serial_println!("[ok]");
    asm::exit_qemu(asm::QemuExitCode::Success);
    loop {}
}

fn should_panic() {
    assert_eq!(0, 1);
}
