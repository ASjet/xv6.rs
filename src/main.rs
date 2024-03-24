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
use rv64::insn::{self, m, s, u, RegisterRO, RegisterRW};
use xv6::arch;
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

#[entry]
fn start() -> ! {
    let hart_id = m::mhartid.read();
    unsafe {
        // Disable paging for now.
        s::satp.write(0);

        // Delegate all interrupts and exceptions to supervisor mode.
        m::medeleg.write(0xffff);
        m::mideleg.write(0xffff);
        s::sie.set_mask(s::SIE_SEIE | s::SIE_STIE | s::SIE_SSIE);

        // Configure Physical Memory Protection to give supervisor mode
        // access to all of physical memory.
        m::pmpaddr0.write(0x3fffffffffffff);
        m::pmpcfg0.write(0xf);

        init_timer_interrupt(hart_id);

        // set M Previous Privilege mode to Supervisor, for mret.
        m::mstatus.w_mpp(insn::PrivilegeLevel::S);
        // set M Exception Program Counter to main, for mret.
        m::mepc.write(main as usize);
        // Keep each CPU's hartid in its tp register, for cpuid().
        u::tp.write(hart_id);
    }
    m::mret();
    halt();
}

/// `start()` jumps here in S mode on all CPUs.
extern "C" fn main() -> ! {
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

static mut TIMER_SCRATCH: [[u64; 5]; 8] = [[0; 5]; 8];

unsafe fn init_timer_interrupt(hart_id: usize) {
    extern "C" {
        fn timer_vec();
    }

    unsafe {
        let interval = 1000000; // cycles; about 1/10th second in qemu.

        // ask the CLINT for a timer interrupt.
        let mtimecmp_ptr = arch::def::clint_mtimecmp(hart_id as u64) as *mut u64;
        *(mtimecmp_ptr) = *(arch::def::CLINT_MTIME as *const u64) + interval;

        let scratch = &mut TIMER_SCRATCH[hart_id];
        scratch[3] = mtimecmp_ptr as u64;
        scratch[4] = interval;
        m::mscratch.write(scratch.as_ptr() as usize);

        m::mtvec.write(timer_vec as usize);

        // Enable machine-mode interrupts.
        m::mstatus.set_mask(m::MSTATUS_MIE);

        // Enable machine-mode timer interrupts.
        m::mie.set_mask(m::MIE_MTIE);
    }
}

#[export_name = "_mp_hook"]
pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
    hartid == 0
}

#[export_name = "_setup_interrupts"]
pub extern "Rust" fn setup_interrupts() {
    // Do nothing and delegate to `start()`
}
