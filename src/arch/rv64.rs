use rv64::insn::{s, u, RegisterRW};

pub mod def;
pub mod interrupt;
pub mod trap;
pub mod vm;

#[inline]
pub fn is_intr_on() -> bool {
    (s::sstatus.read() & s::SSTATUS_SIE.mask()) != 0
}

#[inline]
pub fn intr_on() {
    unsafe { s::sstatus.set_mask(s::SSTATUS_SIE) };
}

#[inline]
pub fn intr_off() {
    unsafe { s::sstatus.clear_mask(s::SSTATUS_SIE) };
}

#[inline]
pub fn halt() -> ! {
    loop {
        unsafe { core::arch::riscv64::wfi() };
    }
}

#[inline]
pub fn cpuid() -> usize {
    u::tp.read()
}
