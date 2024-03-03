use core::panic::PanicInfo;

use crate::arch;
use crate::{serial_print, serial_println};

pub fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    arch::exit_qemu(arch::QemuExitCode::Failure);
    loop {}
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) -> () {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
        serial_println!("[ok]");
    }

    arch::exit_qemu(arch::QemuExitCode::Success);
}
