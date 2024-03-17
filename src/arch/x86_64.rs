#![allow(dead_code)]

pub mod gdt;
pub mod interrupts;
pub mod vm;

mod port;
pub use port::*;

use crate::{dmesg, println};
use bootloader::BootInfo;
use core::fmt::{Arguments, Write};

#[inline]
pub fn halt() -> ! {
    dmesg!("system halt");
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline]
pub fn getcpu() -> u64 {
    let ebx = unsafe { core::arch::x86_64::__cpuid(1).ebx };
    ((ebx >> 24) & 0xff) as u64
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

pub fn init(boot_info: &'static BootInfo) {
    gdt::init();
    dmesg!("GDT initialized");
    interrupts::init_idt();
    dmesg!("IDT initialized");
    interrupts::init_pic();
    dmesg!("PIC initialized");
    vm::init(boot_info);
    dmesg!("VM initialized");
}
