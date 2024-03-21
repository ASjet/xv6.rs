use rv64::insn::{s, RegisterRW};

pub mod def;

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
