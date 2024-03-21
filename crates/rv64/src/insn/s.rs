use super::Mask;
use crate::{csr_reg_ro, csr_reg_rw};

csr_reg_rw!(
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

csr_reg_rw!(
    /// Supervisor Interrupt Pending
    sip
);
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

csr_reg_rw!(
    /// Supervisor Interrupt Enable
    sie
);
pub const SIE_SEIE: Mask = Mask::new(1, 9); // external
pub const SIE_STIE: Mask = Mask::new(1, 5); // timer
pub const SIE_SSIE: Mask = Mask::new(1, 1); // software

csr_reg_rw!(
    /// Supervisor exception program counter, holds the instruction
    /// address to which a return from exception will go.
    sepc
);

csr_reg_rw!(
    /// Supervisor Trap-Vector Base Address low two bits are mode.
    stvec
);

/// use riscv's sv39 page table scheme.
pub const SATP_SV39: Mask = Mask::new(1, 63);
pub const fn make_satp(pagetable: u64) -> u64 {
    SATP_SV39.mask() | (pagetable >> 12)
}
csr_reg_rw!(
    /// Supervisor address translation and protection;
    /// holds the address of the page table.
    satp
);

csr_reg_rw!(
    /// Supervisor Scratch register, for early trap handler
    sscratch
);

csr_reg_ro!(
    /// Supervisor Trap Cause
    scause
);

csr_reg_ro!(
    /// Supervisor Trap Value
    stval
);
