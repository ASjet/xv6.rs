use super::RegisterRW;
use crate::{csr_reg_ro, csr_reg_rw, csr_set_clear, vm::PA_PPN, BitFlag, PrivilegeLevel};
use int_enum::IntEnum;

/*            Supervisor Trap Setup            */

csr_reg_rw!(
    /// Supervisor status register
    sstatus, Sstatus
);
impl sstatus {
    pub const SD: BitFlag = BitFlag::new(1, 63);
    pub const UXL: BitFlag = BitFlag::new(2, 32);
    pub const MXR: BitFlag = BitFlag::new(1, 19);
    pub const SUM: BitFlag = BitFlag::new(1, 18);
    pub const XS: BitFlag = BitFlag::new(2, 15);
    pub const FS: BitFlag = BitFlag::new(2, 13);
    pub const VS: BitFlag = BitFlag::new(2, 9);
    pub const SPP: BitFlag = BitFlag::new(1, 8); // Previous mode, 1=Supervisor, 0=User
    pub const UBE: BitFlag = BitFlag::new(1, 6);
    pub const SPIE: BitFlag = BitFlag::new(1, 5); // Supervisor Previous Interrupt Enable
    pub const SIE: BitFlag = BitFlag::new(1, 1); // Supervisor Interrupt Enable
}
csr_set_clear!(sstatus, set_sie, clear_sie, sstatus::SIE);
impl Sstatus {
    /// Read `sstatus.SPP`
    #[inline]
    pub fn spp(&self) -> PrivilegeLevel {
        unsafe { PrivilegeLevel::try_from(sstatus::SPP.read(self.0) as u8).unwrap_unchecked() }
    }

    /// Read `sstatus.SIE`
    #[inline]
    pub fn sie(&self) -> bool {
        sstatus::SIE.mask(self.0) == 1
    }
}

csr_reg_rw!(
    /// Supervisor interrupt-enable register
    sie
);
impl sie {
    pub const SEIE: BitFlag = BitFlag::new(1, 9); // external
    pub const STIE: BitFlag = BitFlag::new(1, 5); // timer
    pub const SSIE: BitFlag = BitFlag::new(1, 1); // software
}

csr_reg_rw!(
    /// Supervisor trap handler base address
    stvec, Stvec
);
/// Trap mode
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TrapModeS {
    Direct = 0,
    Vectored = 1,
}
impl Stvec {
    /// Returns the trap-vector base-address
    #[inline]
    pub fn address(&self) -> usize {
        self.0 - (self.0 & 0b11)
    }

    /// Returns the trap-vector mode
    #[inline]
    pub fn trap_mode(&self) -> Option<TrapModeS> {
        let mode = self.0 & 0b11;
        match mode {
            0 => Some(TrapModeS::Direct),
            1 => Some(TrapModeS::Vectored),
            _ => None,
        }
    }
}

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

csr_reg_rw!(
    /// Supervisor trap cause
    scause, Scause
);
impl scause {
    pub const INTERRUPT: BitFlag = BitFlag::new(1, 63); // 1: interrupt, 0: exception
    pub const EXCEPTION: BitFlag = BitFlag::new(63, 0);
}
#[derive(Debug)]
pub enum ScauseInterrupt {
    Reserved(usize),
    PlatformUse(usize),
    SupervisorSoftwareInterrupt,
    SupervisorTimerInterrupt,
    SupervisorExternalInterrupt,
    CounterOverflowInterrupt,
}
#[derive(Debug)]
pub enum ScauseException {
    Reserved(usize),
    CustomUse(usize),
    InsnAddrMisaligned,
    InsnAccessFault,
    IllegalInsn,
    Breakpoint,
    LoadAddrMisaligned,
    LoadAccessFault,
    StoreAddrMisaligned,
    StoreAccessFault,
    EnvCallFromU,
    EnvCallFromS,
    InsnPageFault,
    LoadPageFault,
    StorePageFault,
    SoftwareCheck,
    HardwareError,
}
impl Scause {
    pub fn is_interrupt(&self) -> bool {
        scause::INTERRUPT.read(self.0) == 1
    }

    pub fn is_exception(&self) -> bool {
        scause::INTERRUPT.read(self.0) == 0
    }

    pub fn interrupt(&self) -> ScauseInterrupt {
        let except = scause::EXCEPTION.read(self.0);
        match except {
            0 => ScauseInterrupt::Reserved(except),
            1 => ScauseInterrupt::SupervisorSoftwareInterrupt,
            2..=4 => ScauseInterrupt::Reserved(except),
            5 => ScauseInterrupt::SupervisorTimerInterrupt,
            6..=8 => ScauseInterrupt::Reserved(except),
            9 => ScauseInterrupt::SupervisorExternalInterrupt,
            10..=12 => ScauseInterrupt::Reserved(except),
            13 => ScauseInterrupt::CounterOverflowInterrupt,
            14..=15 => ScauseInterrupt::Reserved(except),
            _ => ScauseInterrupt::PlatformUse(except),
        }
    }

    pub fn exception(&self) -> ScauseException {
        let except = scause::EXCEPTION.read(self.0);
        match except {
            0 => ScauseException::InsnAddrMisaligned,
            1 => ScauseException::InsnAccessFault,
            2 => ScauseException::IllegalInsn,
            3 => ScauseException::Breakpoint,
            4 => ScauseException::LoadAddrMisaligned,
            5 => ScauseException::LoadAccessFault,
            6 => ScauseException::StoreAddrMisaligned,
            7 => ScauseException::StoreAccessFault,
            8 => ScauseException::EnvCallFromU,
            9 => ScauseException::EnvCallFromS,
            10..=11 => ScauseException::Reserved(except),
            12 => ScauseException::InsnPageFault,
            13 => ScauseException::LoadPageFault,
            14..=14 => ScauseException::Reserved(except),
            15 => ScauseException::StorePageFault,
            16..=17 => ScauseException::Reserved(except),
            18 => ScauseException::SoftwareCheck,
            19 => ScauseException::HardwareError,
            20..=23 => ScauseException::Reserved(except),
            24..=31 => ScauseException::CustomUse(except),
            32..=47 => ScauseException::Reserved(except),
            48..=63 => ScauseException::CustomUse(except),
            _ => ScauseException::Reserved(except),
        }
    }
}

csr_reg_ro!(
    /// Supervisor bad address or instruction
    stval
);

csr_reg_rw!(
    /// Supervisor interrupt pending
    sip
);
impl sip {
    pub const SEIP: BitFlag = BitFlag::new(1, 9); // external
    pub const STIP: BitFlag = BitFlag::new(1, 5); // timer
    pub const SSIP: BitFlag = BitFlag::new(1, 1); // software
}
csr_set_clear!(sip, set_ssip, clear_ssip, sip::SSIP);

/*            Supervisor Protection and Translation            */

csr_reg_rw!(
    /// Supervisor address translation and protection
    satp
);
impl satp {
    pub const MODE: BitFlag = BitFlag::new(4, 60);
    pub const ASID: BitFlag = BitFlag::new(16, 44);
    pub const PPN: BitFlag = BitFlag::new(44, 0);
}

#[derive(Debug, IntEnum)]
#[repr(u8)]
pub enum SatpMode {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10,
    Sv64 = 11,
}

impl satp {
    #[inline]
    pub fn mode(&self) -> Option<SatpMode> {
        SatpMode::try_from(self.read_mask(satp::MODE) as u8).ok()
    }

    #[inline]
    pub unsafe fn set(&self, mode: SatpMode, asid: usize, pa: usize) {
        self.write(self.make(mode, asid, pa));
    }

    #[inline]
    pub unsafe fn make(&self, mode: SatpMode, asid: usize, pa: usize) -> usize {
        (satp::MODE.make(mode as usize))
            | (satp::ASID.make(asid))
            | (satp::PPN.make(PA_PPN.read(pa)))
    }
}
