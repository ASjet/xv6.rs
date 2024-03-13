#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(xv6::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

pub fn main(bootinfo: &'static BootInfo) -> ! {
    xv6::init(bootinfo);
    test_main();
    xv6::arch::halt();
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    xv6::test::panic_handler(info)
}

#[test_case]
fn basic_alloc() {
    let b1 = Box::new(42);
    let b2 = Box::new(16);
    assert_eq!(*b1, 42);
    assert_eq!(*b2, 16);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), n * (n - 1) / 2);
}

#[test_case]
fn many_alloc() {
    for i in 0..xv6::arch::vm::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
