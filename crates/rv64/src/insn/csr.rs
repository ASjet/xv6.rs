use super::{Mask, Register};
use core::arch::asm;
use int_enum::IntEnum;

macro_rules! csr_reg {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
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

#[derive(Debug, IntEnum)]
#[repr(u8)]
pub enum PrivilegeLevel {
    U = 0b00,
    S = 0b01,
    /*  0b10 is reserved */
    M = 0b11,
}

csr_reg!(
    /// Current hart id
    mhartid
);

csr_reg!(
    /// Machine-mode Status Register
    mstatus
);
impl mstatus {
    /// Read mstatus.MPP
    #[inline]
    pub fn r_mpp(&self) -> PrivilegeLevel {
        PrivilegeLevel::try_from(self.read_mask(MSTATUS_MPP) as u8).unwrap()
    }

    /// Write mstatus.MPP
    #[inline]
    pub unsafe fn w_mpp(&self, l: PrivilegeLevel) {
        unsafe { self.write_mask(MSTATUS_MPP, l as u64) }
    }
}
pub const MSTATUS_TSR: Mask = Mask::new(1, 22);
pub const MSTATUS_TW: Mask = Mask::new(1, 21);
pub const MSTATUS_TVM: Mask = Mask::new(1, 20);
pub const MSTATUS_MXR: Mask = Mask::new(1, 19);
pub const MSTATUS_SUM: Mask = Mask::new(1, 18);
pub const MSTATUS_MPRV: Mask = Mask::new(1, 17);
pub const MSTATUS_XS: Mask = Mask::new(2, 15);
pub const MSTATUS_FS: Mask = Mask::new(2, 13);
pub const MSTATUS_MPP: Mask = Mask::new(2, 11); // Machine-mode Previous Privilege
pub const MSTATUS_VS: Mask = Mask::new(2, 9);
pub const MSTATUS_SPP: Mask = Mask::new(1, 8); // Supervisor Previous Privilege
pub const MSTATUS_MPIE: Mask = Mask::new(1, 7);
pub const MSTATUS_UBE: Mask = Mask::new(1, 6);
pub const MSTATUS_SPIE: Mask = Mask::new(1, 5); // Supervisor Previous Interrupt Enable
pub const MSTATUS_MIE: Mask = Mask::new(1, 3); // Machine-mode Interrupt Enable
pub const MSTATUS_SIE: Mask = Mask::new(1, 1); // Supervisor Interrupt Enable

csr_reg!(
    /// Supervisor Status Register
    sstatus
);
pub const SSTATUS_SD: Mask = Mask::new(1, 63);
pub const SSTATUS_UXL: Mask = Mask::new(2, 32);
pub const SSTATUS_MXR: Mask = Mask::new(1, 19);
pub const SSTATUS_SUM: Mask = Mask::new(1, 18);
pub const SSTATUS_XS: Mask = Mask::new(2, 15);
pub const SSTATUS_FS: Mask = Mask::new(2, 13);
pub const SSTATUS_VS: Mask = Mask::new(2, 9);
pub const SSTATUS_SPP: Mask = Mask::new(1, 8); // Previous mode, 1=Supervisor, 0=User
pub const SSTATUS_UBE: Mask = Mask::new(1, 6);
pub const SSTATUS_SPIE: Mask = Mask::new(1, 5); // Supervisor Previous Interrupt Enable
pub const SSTATUS_SIE: Mask = Mask::new(1, 1); // Supervisor Interrupt Enable

csr_reg!(
    /// Machine-mode Interrupt Pending
    mip
);
pub const MIP_MEIP: Mask = Mask::new(1, 11); // external
pub const MIP_SEIP: Mask = Mask::new(1, 9); // external
pub const MIP_MTIP: Mask = Mask::new(1, 7); // timer
pub const MIP_STIP: Mask = Mask::new(1, 5); // timer
pub const MIP_MSIP: Mask = Mask::new(1, 3); // software
pub const MIP_SSIP: Mask = Mask::new(1, 1); // software

csr_reg!(
    /// Supervisor Interrupt Pending
    sip
);
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

csr_reg!(
    /// Machine-mode Interrupt Enable
    mie
);
pub const MIE_SEIE: Mask = Mask::new(1, 11); // external
pub const MIE_MTIE: Mask = Mask::new(1, 9); // timer
pub const MIE_STIE: Mask = Mask::new(1, 7); // timer
pub const MIE_MSIE: Mask = Mask::new(1, 5); // software
pub const MIE_SSIE: Mask = Mask::new(1, 3); // software

csr_reg!(
    /// Supervisor Interrupt Enable
    sie
);
pub const SIE_SEIE: Mask = Mask::new(1, 9); // external
pub const SIE_STIE: Mask = Mask::new(1, 5); // timer
pub const SIE_SSIE: Mask = Mask::new(1, 1); // software

csr_reg!(
    /// Machine exception program counter, holds the instruction
    /// address to which a return from exception will go.
    mepc
);

csr_reg!(
    /// Supervisor exception program counter, holds the instruction
    /// address to which a return from exception will go.
    sepc
);

csr_reg!(
    /// Machine Exception Delegation
    medeleg
);

csr_reg!(
    /// Machine Interrupt Delegation
    mideleg
);

csr_reg!(
    /// Machine-mode interrupt vector
    mtvec
);

csr_reg!(
    /// Supervisor Trap-Vector Base Address low two bits are mode.
    stvec
);

csr_reg!(pmpcfg0);

csr_reg!(pmpaddr0);

/// use riscv's sv39 page table scheme.
pub const SATP_SV39: Mask = Mask::new(1, 63);
pub const fn make_satp(pagetable: u64) -> u64 {
    SATP_SV39.mask() | (pagetable >> 12)
}
csr_reg!(
    /// Supervisor address translation and protection;
    /// holds the address of the page table.
    satp
);

csr_reg!(
    /// Machine-mode Scratch register, for early trap handler
    mscratch
);

csr_reg!(
    /// Supervisor Scratch register, for early trap handler
    sscratch
);

csr_reg!(
    /// Supervisor Trap Cause
    scause
);

csr_reg!(
    /// Supervisor Trap Value
    stval
);

csr_reg!(
    /// Machine-mode Counter-Enable
    mcounteren
);

csr_reg!(
    /// machine-mode cycle counter
    time
);
