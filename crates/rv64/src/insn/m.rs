use super::{Mask, PrivilegeLevel, RegisterRW};
use crate::{csr_reg_ro, csr_reg_rw};

/*            Machine Information Register            */

csr_reg_ro!(
    /// Vendor ID
    mvendorid
);

csr_reg_ro!(
    /// Architecture ID
    marchid
);

csr_reg_ro!(
    /// Implementation ID
    mimpid
);

csr_reg_ro!(
    /// Hardware thread ID
    mhartid
);

csr_reg_ro!(
    /// Pointer to configuration data structure
    mconfigptr
);

/*            Machine Trap Setup            */

csr_reg_rw!(
    /// Machine status register
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
    /// ISA and extensions
    misa
);

csr_reg_rw!(
    /// Machine exception delegation register
    medeleg
);

csr_reg_rw!(
    /// Machine interrupt delegation register
    mideleg
);

csr_reg_rw!(
    /// Machine interrupt-enable register
    mie
);
pub const MIE_SEIE: Mask = Mask::new(1, 11); // external
pub const MIE_MTIE: Mask = Mask::new(1, 9); // timer
pub const MIE_STIE: Mask = Mask::new(1, 7); // timer
pub const MIE_MSIE: Mask = Mask::new(1, 5); // software
pub const MIE_SSIE: Mask = Mask::new(1, 3); // software

csr_reg_rw!(
    /// Machine trap-handler base address
    mtvec
);

csr_reg_rw!(
    /// Machine counter enable
    mcounteren
);

/*            Machine Trap Handling            */

csr_reg_rw!(
    /// Scratch register for machine trap handlers
    mscratch
);

csr_reg_rw!(
    /// Machine exception program counter
    mepc
);

csr_reg_rw!(
    /// Machine trap cause
    mcause
);

csr_reg_rw!(
    /// Machine bad address or instruction
    mtval
);

csr_reg_rw!(
    /// Machine interrupt pending
    mip
);
pub const MIP_MEIP: Mask = Mask::new(1, 11); // external
pub const MIP_SEIP: Mask = Mask::new(1, 9); // external
pub const MIP_MTIP: Mask = Mask::new(1, 7); // timer
pub const MIP_STIP: Mask = Mask::new(1, 5); // timer
pub const MIP_MSIP: Mask = Mask::new(1, 3); // software
pub const MIP_SSIP: Mask = Mask::new(1, 1); // software

csr_reg_rw!(
    /// Machine trap instruction (transformed)
    mtinst
);

csr_reg_rw!(
    /// Machine bad guest physical address
    mtval2
);

/*            Machine Configuration            */

csr_reg_rw!(
    /// Machine environment configuration register
    menvcfg
);

csr_reg_rw!(
    /// Machine security configuration register
    mseccfg
);

/*            Machine Memory Protection            */

csr_reg_rw!(
    /// Physical memory protection configuration
    pmpcfg0
);
// ...

csr_reg_rw!(
    /// Physical memory protection address register
    pmpaddr0
);
// ...

/*            Machine Non-Maskable Interrupt Handling            */

csr_reg_rw!(
    /// Resumable NMI scratch register
    mnscratch
);

csr_reg_rw!(
    /// Resumable NMI program counter
    mnepc
);

csr_reg_rw!(
    /// Resumable NMI cause
    mncause
);

csr_reg_rw!(
    /// Resumable NMI status
    mnstatus
);

/*            Machine Counter/Timers            */

csr_reg_rw!(
    /// Machine cycle counter
    mcycle
);

csr_reg_rw!(
    /// Machine instructions-retired counter
    minstret
);

/*            Machine Counter Setup            */

csr_reg_rw!(
    /// Machine counter-inhibit register
    mcountinhibit
);
