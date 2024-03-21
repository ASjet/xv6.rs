use super::{Mask, PrivilegeLevel, RegisterRW};
use crate::{csr_reg_ro, csr_reg_rw};

csr_reg_ro!(
    /// Current hart id
    mhartid
);

csr_reg_rw!(
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

csr_reg_rw!(
    /// Machine-mode Interrupt Pending
    mip
);
pub const MIP_MEIP: Mask = Mask::new(1, 11); // external
pub const MIP_SEIP: Mask = Mask::new(1, 9); // external
pub const MIP_MTIP: Mask = Mask::new(1, 7); // timer
pub const MIP_STIP: Mask = Mask::new(1, 5); // timer
pub const MIP_MSIP: Mask = Mask::new(1, 3); // software
pub const MIP_SSIP: Mask = Mask::new(1, 1); // software

csr_reg_rw!(
    /// Machine-mode Interrupt Enable
    mie
);
pub const MIE_SEIE: Mask = Mask::new(1, 11); // external
pub const MIE_MTIE: Mask = Mask::new(1, 9); // timer
pub const MIE_STIE: Mask = Mask::new(1, 7); // timer
pub const MIE_MSIE: Mask = Mask::new(1, 5); // software
pub const MIE_SSIE: Mask = Mask::new(1, 3); // software

csr_reg_rw!(
    /// Machine exception program counter, holds the instruction
    /// address to which a return from exception will go.
    mepc
);

csr_reg_rw!(
    /// Machine Exception Delegation
    medeleg
);

csr_reg_rw!(
    /// Machine Interrupt Delegation
    mideleg
);

csr_reg_rw!(
    /// Machine-mode interrupt vector
    mtvec
);

csr_reg_rw!(pmpcfg0);

csr_reg_rw!(pmpaddr0);

csr_reg_rw!(
    /// Machine-mode Scratch register, for early trap handler
    mscratch
);

csr_reg_rw!(
    /// Machine-mode Counter-Enable
    mcounteren
);
