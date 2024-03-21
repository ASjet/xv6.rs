use super::Mask;
use crate::{csr_reg_ro, csr_reg_rw};
use core::arch::asm;

#[inline]
pub fn sfence_vma() {
    unsafe { asm!("sfence.vma zero, zero") };
}

/*            Supervisor Trap Setup            */

csr_reg_rw!(
    /// Supervisor status register
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
    /// Supervisor interrupt-enable register
    sie
);
pub const SIE_SEIE: Mask = Mask::new(1, 9); // external
pub const SIE_STIE: Mask = Mask::new(1, 5); // timer
pub const SIE_SSIE: Mask = Mask::new(1, 1); // software

csr_reg_rw!(
    /// Supervisor trap handler base address
    stvec
);

csr_reg_rw!(
    /// Supervisor counter enable
    scounteren
);

/*            Supervisor Configuration            */

csr_reg_rw!(
    /// Supervisor environment configuration register
    senvcfg
);

/*            Supervisor Trap Handling            */

csr_reg_rw!(
    /// Scratch register for supervisor trap handlers
    sscratch
);

csr_reg_rw!(
    /// Supervisor exception program counter
    sepc
);

csr_reg_ro!(
    /// Supervisor trap cause
    scause
);

csr_reg_ro!(
    /// Supervisor bad address or instruction
    stval
);

csr_reg_rw!(
    /// Supervisor interrupt pending
    sip
);
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

/*            Supervisor Protection and Translation            */

csr_reg_rw!(
    /// Supervisor address translation and protection
    satp
);
/// use riscv's sv39 page table scheme.
pub const SATP_SV39: Mask = Mask::new(1, 63);
pub const fn make_satp(pagetable: u64) -> u64 {
    SATP_SV39.mask() | (pagetable >> 12)
}
