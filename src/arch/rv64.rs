use rv64::insn::{csr, Register};

pub mod def;

#[inline]
pub fn is_intr_on() -> bool {
    (csr::sstatus.read() & csr::SSTATUS_SIE.mask()) != 0
}

#[inline]
pub fn intr_on() {
    unsafe { csr::sstatus.set_mask(csr::SSTATUS_SIE) };
}

#[inline]
pub fn intr_off() {
    unsafe { csr::sstatus.clear_mask(csr::SSTATUS_SIE) };
}
