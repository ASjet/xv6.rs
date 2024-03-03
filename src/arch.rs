#![allow(dead_code)]

pub mod gdt;
pub mod interrupt;

mod port;
pub use port::*;

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
