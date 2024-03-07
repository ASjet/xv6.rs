#![allow(dead_code)]

pub mod gdt;
pub mod interrupts;

mod port;
pub use port::*;

use crate::println;
use core::fmt::{Arguments, Write};

#[inline]
pub fn halt() -> ! {
    println!("[{:12.6}] system halt", interrupts::ticks());
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
    println!("[{:12.6}] GDT initialized", interrupts::ticks());
    interrupts::init_idt();
    println!("[{:12.6}] IDT initialized", interrupts::ticks());
    interrupts::init_pic();
    println!("[{:12.6}] PIC initialized", interrupts::ticks());
}
