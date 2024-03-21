#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

use core::panic::PanicInfo;
use riscv_rt::entry;
use rv64::insn::{m, u, RegisterRO, RegisterRW};
use xv6::println;

extern "C" {
    static _sheap: u8;
    static _heap_size: u8;
    static _max_hart_id: u8;
}

#[inline]
fn halt() -> ! {
    loop {
        unsafe { core::arch::riscv64::wfi() };
    }
}

fn cpuid() -> u64 {
    u::tp.read()
}

/// This will only be called on hart 0,
/// while other harts will be in `wfi` with default `_mp_hook` implementation.
#[entry]
fn main() -> ! {
    println!("hello, world! hartid: {}/{}\n", cpuid(), unsafe {
        ((&_max_hart_id) as *const u8) as usize
    });
    unsafe {
        let heap_bottom = &_sheap as *const u8 as usize;
        let heap_size = &_heap_size as *const u8 as usize;
        println!(
            "heap_bottom: {:x}, heap_size: {:x}\n",
            heap_bottom, heap_size
        );
    }
    halt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt();
}

#[export_name = "_mp_hook"]
pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
    assert_eq!(hartid as u64, m::mhartid.read());
    unsafe { u::tp.write(hartid as u64) };
    println!("{}\n", hartid);
    if hartid == 0 {
        return true;
    } else {
        halt();
    }
}
