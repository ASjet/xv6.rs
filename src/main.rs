#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

use core::panic::PanicInfo;
use riscv_rt::entry;

#[inline]
fn halt() -> ! {
    loop {
        unsafe { core::arch::riscv64::wfi() };
    }
}

/// This will only be called on hart 0,
/// while other harts will be in `wfi` with default `_mp_hook` implementation.
#[entry]
fn main() -> ! {
    // println!("\nHello, world!\n");
    halt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt();
}

// #[export_name = "_mp_hook"]
// pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
//     // ...
//     todo!()
// }
