use rv64::reg::{self, RegisterRW};

pub mod def;
pub mod interrupt;
pub mod trampoline;
pub mod trap;
pub mod vm;

#[inline]
pub fn is_intr_on() -> bool {
    reg::sstatus.read().sie()
}

#[inline]
pub fn intr_on() {
    unsafe { reg::sstatus.set_mask(reg::sstatus::SIE) };
}

#[inline]
pub fn intr_off() {
    unsafe { reg::sstatus.clear_mask(reg::sstatus::SIE) };
}

#[inline]
pub fn halt() -> ! {
    loop {
        unsafe { core::arch::riscv64::wfi() };
    }
}

/// Must be called with interrupts disabled,
/// to prevent race with process being moved
/// to a different CPU.
#[inline]
pub fn cpuid() -> usize {
    reg::tp.read()
}
