//! the riscv Platform Level Interrupt Controller (PLIC).

use crate::arch::def;
use crate::io::{BaseIO, ScratchIO};

const PLIC_BASE: BaseIO<u32> = BaseIO::new(def::PLIC as usize);
const PLIC_MENABLE: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x2000, 0x100);
const PLIC_SENABLE: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x2080, 0x100);
const PLIC_MPRIORITY: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x200000, 0x2000);
const PLIC_SPRIORITY: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x201000, 0x2000);
const PLIC_MCLAIM: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x200004, 0x2000);
const PLIC_SCLAIM: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x201004, 0x2000);

pub fn plic_init() {
    // set desired IRQ priorities non-zero (otherwise disabled).
    PLIC_BASE.offset((def::UART0_IRQ * 4) as usize).write(1);
    PLIC_BASE.offset((def::VIRTIO0_IRQ * 4) as usize).write(1);
}

pub fn plic_init_hart() {
    let hart = crate::arch::cpuid();
    // set uart's enable bit for this hart's S-mode.
    PLIC_SENABLE
        .index(hart)
        .write((1 << def::UART0_IRQ) | (1 << def::VIRTIO0_IRQ));

    // set this hart's S-mode priority threshold to 0.
    PLIC_SPRIORITY.index(hart).write(0);
}

/// ask the PLIC what interrupt we should serve.
pub fn plic_claim(hart: usize) -> u32 {
    PLIC_SCLAIM.index(hart).read()
}

/// tell the PLIC we've served this IRQ.
pub fn plic_complete(hart: usize, irq: u32) {
    PLIC_SCLAIM.index(hart).write(irq);
}
