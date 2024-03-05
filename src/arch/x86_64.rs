#![allow(dead_code)]

pub mod gdt;
pub mod interrupts;

mod port;
pub use port::*;

#[inline]
pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
