#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use xv6::asm;
use xv6::serial_println;
use xv6::test::Testable;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
pub fn panic(_: &PanicInfo) -> ! {
    serial_println!("[ok]");
    asm::exit_qemu(asm::QemuExitCode::Success);
    loop {}
}

fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
        serial_println!("[test did not panic]");
        asm::exit_qemu(asm::QemuExitCode::Failure);
    }
    asm::exit_qemu(asm::QemuExitCode::Success);
}

#[test_case]
fn should_panic() {
    assert_eq!(0, 1);
}
