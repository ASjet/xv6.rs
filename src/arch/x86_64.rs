#![allow(dead_code)]

pub mod gdt;
pub mod interrupts;

mod port;
pub use port::*;

use core::fmt::{Arguments, Write};
use x86_64::instructions::interrupts::without_interrupts;

#[inline]
pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline]
pub fn scan_code() -> u8 {
    PortIndex::ScanCode.read()
}

#[doc(hidden)]
pub fn serial_print(args: Arguments) {
    without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    })
}
