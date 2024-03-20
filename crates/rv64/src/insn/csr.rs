use super::{Mask, Register};
use core::arch::asm;

macro_rules! csr_reg {
    ($reg:ident) => {
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl Register for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe { asm!(concat!("csrr {}, ", stringify!($reg)), out(reg) r) };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe { asm!(concat!("csrw ", stringify!($reg), ", {}"), in(reg) x) };
            }
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

csr_reg!(mhartid);

// Machine mode status
csr_reg!(mstatus);
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

#[inline]
pub fn r_mstatus_mpp() -> PrivilegeLevel {
    PrivilegeLevel::from_u64(mstatus.read_mask(MSTATUS_MPP))
}

#[inline]
pub unsafe fn w_mstatus_mpp(l: PrivilegeLevel) {
    unsafe { mstatus.write_mask(MSTATUS_MPP, l as u64) }
}

// Supervisor mode status
csr_reg!(sstatus);
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

// Machine Interrupt Pending
csr_reg!(mip);
pub const MIP_MEIP: Mask = Mask::new(1, 11); // external
pub const MIP_SEIP: Mask = Mask::new(1, 9); // external
pub const MIP_MTIP: Mask = Mask::new(1, 7); // timer
pub const MIP_STIP: Mask = Mask::new(1, 5); // timer
pub const MIP_MSIP: Mask = Mask::new(1, 3); // software
pub const MIP_SSIP: Mask = Mask::new(1, 1); // software

// Supervisor Interrupt Pending
csr_reg!(sip);
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

// Machine Interrupt Enable
csr_reg!(mie);
pub const MIE_SEIE: Mask = Mask::new(1, 11); // external
pub const MIE_MTIE: Mask = Mask::new(1, 9); // timer
pub const MIE_STIE: Mask = Mask::new(1, 7); // timer
pub const MIE_MSIE: Mask = Mask::new(1, 5); // software
pub const MIE_SSIE: Mask = Mask::new(1, 3); // software

// Supervisor Interrupt Enable
csr_reg!(sie);
pub const SIE_SEIE: Mask = Mask::new(1, 9); // external
pub const SIE_STIE: Mask = Mask::new(1, 5); // timer
pub const SIE_SSIE: Mask = Mask::new(1, 1); // software

csr_reg!(mepc);

csr_reg!(sepc);

csr_reg!(medeleg);

csr_reg!(mideleg);

csr_reg!(mtvec);

csr_reg!(stvec);
