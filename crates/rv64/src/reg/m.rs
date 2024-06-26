use super::RegisterRW;
use crate::{csr_reg_ro, csr_reg_rw, csr_set_clear, BitFlag, PrivilegeLevel};

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
    pub const TSR: BitFlag = BitFlag::new(1, 22);
    pub const TW: BitFlag = BitFlag::new(1, 21);
    pub const TVM: BitFlag = BitFlag::new(1, 20);
    pub const MXR: BitFlag = BitFlag::new(1, 19);
    pub const SUM: BitFlag = BitFlag::new(1, 18);
    pub const MPRV: BitFlag = BitFlag::new(1, 17);
    pub const XS: BitFlag = BitFlag::new(2, 15);
    pub const FS: BitFlag = BitFlag::new(2, 13);
    pub const MPP: BitFlag = BitFlag::new(2, 11); // Machine-mode Previous Privilege
    pub const VS: BitFlag = BitFlag::new(2, 9);
    pub const SPP: BitFlag = BitFlag::new(1, 8); // Supervisor Previous Privilege
    pub const MPIE: BitFlag = BitFlag::new(1, 7);
    pub const UBE: BitFlag = BitFlag::new(1, 6);
    pub const SPIE: BitFlag = BitFlag::new(1, 5); // Supervisor Previous Interrupt Enable
    pub const MIE: BitFlag = BitFlag::new(1, 3); // Machine-mode Interrupt Enable
    pub const SIE: BitFlag = BitFlag::new(1, 1); // Supervisor Interrupt Enable

    /// Read mstatus.MPP
    #[inline]
    pub fn r_mpp(&self) -> PrivilegeLevel {
        PrivilegeLevel::try_from(self.read_mask(mstatus::MPP) as u8).unwrap()
    }

    /// Write mstatus.MPP
    #[inline]
    pub unsafe fn w_mpp(&self, l: PrivilegeLevel) {
        unsafe { self.write_mask(mstatus::MPP, l as usize) }
    }
}
csr_set_clear!(mstatus, set_mie, clear_mie, mstatus::MIE);

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
impl mie {
    pub const SEIE: BitFlag = BitFlag::new(1, 11); // external
    pub const MTIE: BitFlag = BitFlag::new(1, 9); // timer
    pub const STIE: BitFlag = BitFlag::new(1, 7); // timer
    pub const MSIE: BitFlag = BitFlag::new(1, 5); // software
    pub const SSIE: BitFlag = BitFlag::new(1, 3); // software
}
csr_set_clear!(mie, set_mtie, clear_mtie, mie::MTIE);
csr_set_clear!(mie, set_msoft, clear_msoft, mie::MSIE);
csr_set_clear!(mie, set_ssoft, clear_ssoft, mie::SSIE);

csr_reg_rw!(
    /// Machine trap-handler base address
    mtvec, Mtvec
);

/// Trap mode
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TrapModeM {
    Direct = 0,
    Vectored = 1,
}

impl Mtvec {
    /// Returns the trap-vector base-address
    #[inline]
    pub fn address(&self) -> usize {
        self.0 - (self.0 & 0b11)
    }

    /// Returns the trap-vector mode
    #[inline]
    pub fn trap_mode(&self) -> Option<TrapModeM> {
        let mode = self.0 & 0b11;
        match mode {
            0 => Some(TrapModeM::Direct),
            1 => Some(TrapModeM::Vectored),
            _ => None,
        }
    }
}

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
    mip, Mip
);
impl mip {
    pub const MEIP: BitFlag = BitFlag::new(1, 11); // external
    pub const SEIP: BitFlag = BitFlag::new(1, 9); // external
    pub const MTIP: BitFlag = BitFlag::new(1, 7); // timer
    pub const STIP: BitFlag = BitFlag::new(1, 5); // timer
    pub const MSIP: BitFlag = BitFlag::new(1, 3); // software
    pub const SSIP: BitFlag = BitFlag::new(1, 1); // software
}
impl Mip {
    #[inline]
    pub fn msoft(&self) -> bool {
        mip::MSIP.read(self.0) != 0
    }

    #[inline]
    pub fn ssoft(&self) -> bool {
        mip::SSIP.read(self.0) != 0
    }
}

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
