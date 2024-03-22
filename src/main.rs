#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

//! ## Boot sequence in riscv64:
//!
//! 1. Firmware:
//!     1. Setup SBI arguments:
//!         1. `a0`: value of current `mhartid`
//!         2. `a1`: value at the physical address of `0x1020`(0xbfe00000)
//!         3. `a2`: where `fw_dynamic_info` located
//!     2. Jump to `0x80000000`(`_start`) in M privilege mode
//! 2. `_start`:
//!     1. Init DWARF call frame information
//!     2. Setup stack and frame pointer
//!     3. Clear registers(`MIE`,`MIP`,`X1-X9`,`X13-X31`)
//!     4. Abort hart with `mhartid`(or `a0`) larger than `_max_hart_id`
//!     5. Call `_start_rust(a0, a1, a2)`
//! 3. `_start_rust`:
//!     1. Call `_mp_hook()`, and only hart got a `true` will do the following:
//!         1. Call `__pre_init()`
//!         2. Init RAM
//!             - Copy over .data from flash to RAM
//!             - Zero out .bss(where global and static variables located)
//!     2. Enable fpu in `mstatus` CSR
//!     3. Zero out floating-point registers
//!     4. Call `_setup_interrupts()`
//!     5. Call `main(a0, a1, a2)`.
//!
//! ## Symbols can be override:
//!
//! ### `_max_hart_id`
//!
//! - Default: 0
//! - Override: define in `boot/memory.x` as `_max_hart_id = <value>;`
//!
//! ### `_mp_hook`
//!
//! - Default: halt all harts except hart 0
//! - Override: `#[export_name = "_mp_hook"]`
//!
//! ### `__pre_init`
//!
//! - Default: does nothing
//! - Override: `#[pre_init]`
//!
//! ### `_setup_interrupts`
//!
//! - Default: store `_start_trap` as direct trap mode to `mtvec` CSR, which will do:
//!     1. Save registers to stack
//!     2. Call `_start_trap_rust`, which will call default handlers
//! - Override: `#[export_name = "_setup_interrupts"]`
//!
//! ### `main`
//!
//! This symbol MUST be defined with `#[entry]` attribute.

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

fn cpuid() -> usize {
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
