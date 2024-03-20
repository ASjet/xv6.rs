use core::arch::asm;

pub fn r_mhartid() -> u64 {
    let r: u64;
    unsafe { asm!("csrr {}, mhartid", out(reg) r) };
    r
}
