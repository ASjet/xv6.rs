use super::{Mask, PrivilegeLevel, RegisterRW};
use crate::{csr_reg_ro, csr_reg_rw, naked_insn};
use core::arch::asm;
use int_enum::IntEnum;

#[inline]
pub fn sfence_vma() {
    unsafe { asm!("sfence.vma zero, zero") };
}

naked_insn!(
    /// Return from S mode to U mode and jump to `sepc`
    sret, nomem, nostack
);

/*            Supervisor Trap Setup            */

csr_reg_rw!(
    /// Supervisor status register
    sstatus
);
impl sstatus {
    /// Read sstatus.SPP
    #[inline]
    pub fn r_spp(&self) -> PrivilegeLevel {
        PrivilegeLevel::try_from(self.read_mask(SSTATUS_SPP) as u8).unwrap()
    }

    /// Write sstatus.SPP
    #[inline]
    pub unsafe fn w_spp(&self, l: PrivilegeLevel) {
        unsafe { self.write_mask(SSTATUS_SPP, l as usize) }
    }
}
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

csr_reg_rw!(
    /// Supervisor trap cause
    scause, Scause
);
pub const SCAUSE_INTERRUPT: Mask = Mask::new(1, 63); // 1: interrupt, 0: exception
pub const SCAUSE_EXCEPTION: Mask = Mask::new(63, 0);
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
        SCAUSE_INTERRUPT.get(self.0) == 1
    }

    pub fn is_exception(&self) -> bool {
        SCAUSE_INTERRUPT.get(self.0) == 0
    }

    pub fn interrupt(&self) -> ScauseInterrupt {
        let except = SCAUSE_EXCEPTION.get(self.0);
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
        let except = SCAUSE_EXCEPTION.get(self.0);
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
pub const SIP_SEIP: Mask = Mask::new(1, 9); // external
pub const SIP_STIP: Mask = Mask::new(1, 5); // timer
pub const SIP_SSIP: Mask = Mask::new(1, 1); // software

/*            Supervisor Protection and Translation            */

csr_reg_rw!(
    /// Supervisor address translation and protection
    satp
);
pub const SATP_MODE: Mask = Mask::new(4, 60);
pub const SATP_ASID: Mask = Mask::new(16, 44);
pub const SATP_PPN: Mask = Mask::new(44, 0);

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
        SatpMode::try_from(self.read_mask(SATP_MODE) as u8).ok()
    }

    #[inline]
    pub unsafe fn set(&self, mode: SatpMode, asid: usize, ppn: usize) {
        self.write((SATP_MODE.fill(mode as usize)) | (SATP_ASID.fill(asid)) | (SATP_PPN.fill(ppn)));
    }
}
