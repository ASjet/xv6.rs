use super::Mask;
use core::arch::asm;

macro_rules! csrr {
    ($name:ident, $reg:ident) => {
        #[inline]
        pub fn $name() -> u64 {
            let r: u64;
            unsafe { asm!(concat!("csrr {}, ", stringify!($reg)), out(reg) r) };
            r
        }
    };
}

macro_rules! csrw {
    ($name:ident, $reg:ident) => {
        #[inline]
        pub unsafe fn $name(x: u64) {
            unsafe { asm!(concat!("csrw ", stringify!($reg), ", {}"), in(reg) x) };
        }
    };
}

#[derive(Debug)]
#[repr(u8)]
pub enum PrivilegeLevel {
    U = 0b00,
    S = 0b01,
    R = 0b10, // Reserved
    M = 0b11,
}

impl PrivilegeLevel {
    fn from_u64(x: u64) -> PrivilegeLevel {
        match x & 0b11 {
            0b00 => PrivilegeLevel::U,
            0b01 => PrivilegeLevel::S,
            0b10 => PrivilegeLevel::R,
            0b11 => PrivilegeLevel::M,
            _ => unreachable!(),
        }
    }
}

csrr!(r_mhartid, mhartid);

/// mstatus masks
pub const MSTATUS_TSR: Mask = Mask::new(1, 22);
pub const MSTATUS_TW: Mask = Mask::new(1, 21);
pub const MSTATUS_TVM: Mask = Mask::new(1, 20);
pub const MSTATUS_MXR: Mask = Mask::new(1, 19);
pub const MSTATUS_SUM: Mask = Mask::new(1, 18);
pub const MSTATUS_MPRV: Mask = Mask::new(1, 17);
pub const MSTATUS_XS: Mask = Mask::new(2, 15);
pub const MSTATUS_FS: Mask = Mask::new(2, 13);
pub const MSTATUS_MPP: Mask = Mask::new(2, 11); // previous privilege mode.
pub const MSTATUS_VS: Mask = Mask::new(2, 9);
pub const MSTATUS_SPP: Mask = Mask::new(1, 8);
pub const MSTATUS_MPIE: Mask = Mask::new(1, 7);
pub const MSTATUS_UBE: Mask = Mask::new(1, 6);
pub const MSTATUS_SPIE: Mask = Mask::new(1, 5);
pub const MSTATUS_MIE: Mask = Mask::new(1, 3); // machine-mode interrupt enable.
pub const MSTATUS_SIE: Mask = Mask::new(1, 1); // supervisor-mode interrupt enable.

csrr!(r_mstatus, mstatus);
csrw!(w_mstatus, mstatus);

#[inline]
pub fn r_mstatus_mpp() -> PrivilegeLevel {
    PrivilegeLevel::from_u64(MSTATUS_MPP.get(r_mstatus()))
}

#[inline]
pub unsafe fn w_mstatus_mpp(l: PrivilegeLevel) {
    unsafe { w_mstatus(MSTATUS_MPP.set(r_mstatus(), l as u64)) };
}

/// sstatus masks
pub const SSTATUS_SD: Mask = Mask::new(1, 63);
pub const SSTATUS_UXL: Mask = Mask::new(2, 32);
pub const SSTATUS_MXR: Mask = Mask::new(1, 19);
pub const SSTATUS_SUM: Mask = Mask::new(1, 18);
pub const SSTATUS_XS: Mask = Mask::new(2, 15);
pub const SSTATUS_FS: Mask = Mask::new(2, 13);
pub const SSTATUS_VS: Mask = Mask::new(2, 9);
pub const SSTATUS_SPP: Mask = Mask::new(1, 8);
pub const SSTATUS_UBE: Mask = Mask::new(1, 6);
pub const SSTATUS_SPIE: Mask = Mask::new(1, 5);
pub const SSTATUS_SIE: Mask = Mask::new(1, 1); // supervisor-mode interrupt enable.

csrr!(r_sstatus, sstatus);
csrw!(w_sstatus, sstatus);

/// mip(Machine Interrupt Pending) masks
pub const MIP_MEIP: Mask = Mask::new(1, 11); // external
pub const MIP_SEIP: Mask = Mask::new(1, 9); // external
pub const MIP_MTIP: Mask = Mask::new(1, 7); // timer
pub const MIP_STIP: Mask = Mask::new(1, 5); // timer
pub const MIP_MSIP: Mask = Mask::new(1, 3); // software
pub const MIP_SSIP: Mask = Mask::new(1, 1); // software

csrr!(r_mip, mip);
csrw!(w_mip, mip);

/// sip(Supervisor Interrupt Pending) masks
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

csrr!(r_sip, sip);
csrw!(w_sip, sip);

/// mie(Machine Interrupt Enable) masks
pub const MIE_SEIE: Mask = Mask::new(1, 11); // external
pub const MIE_MTIE: Mask = Mask::new(1, 9); // timer
pub const MIE_STIE: Mask = Mask::new(1, 7); // timer
pub const MIE_MSIE: Mask = Mask::new(1, 5); // software
pub const MIE_SSIE: Mask = Mask::new(1, 3); // software

csrr!(r_mie, mie);
csrw!(w_mie, mie);

/// sie(Supervisor Interrupt Enable) masks
pub const SIE_SEIE: Mask = Mask::new(1, 9); // external
pub const SIE_STIE: Mask = Mask::new(1, 5); // timer
pub const SIE_SSIE: Mask = Mask::new(1, 1); // software

csrr!(r_sie, sie);
csrw!(w_sie, sie);

csrr!(r_mepc, mepc);
csrw!(w_mepc, mepc);

csrr!(r_sepc, sepc);
csrw!(w_sepc, sepc);

csrr!(r_medeleg, medeleg);
csrw!(w_medeleg, medeleg);

csrr!(r_mideleg, mideleg);
csrw!(w_mideleg, mideleg);

csrr!(r_mtvec, mtvec);
csrw!(w_mtvec, mtvec);

csrr!(r_stvec, stvec);
csrw!(w_stvec, stvec);
