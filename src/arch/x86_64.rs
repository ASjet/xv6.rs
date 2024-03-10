#![allow(dead_code)]

pub mod gdt;
pub mod interrupts;
pub mod vm;

mod port;
pub use port::*;

use crate::{dmesg, println};
use core::fmt::{Arguments, Write};

#[inline]
pub fn halt() -> ! {
    dmesg!("system halt");
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
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    })
}

pub fn init() {
    gdt::init();
    dmesg!("GDT initialized");
    interrupts::init_idt();
    dmesg!("IDT initialized");
    interrupts::init_pic();
    dmesg!("PIC initialized")
}
