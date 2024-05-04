//! the riscv Platform Level Interrupt Controller (PLIC).

use crate::arch::def;
use crate::io::{BaseIO, ScratchIO};
use crate::{arch, println, proc};
use rv64::reg::{self, RegisterRW};

const PLIC_BASE: BaseIO<u32> = BaseIO::new(def::PLIC as usize);
const PLIC_MENABLE: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x2000, 0x100);
const PLIC_SENABLE: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x2080, 0x100);
const PLIC_MPRIORITY: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x200000, 0x2000);
const PLIC_SPRIORITY: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x201000, 0x2000);
const PLIC_MCLAIM: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x200004, 0x2000);
const PLIC_SCLAIM: ScratchIO<u32> = ScratchIO::new(def::PLIC as usize + 0x201004, 0x2000);

pub fn init() {
    // set desired IRQ priorities non-zero (otherwise disabled).
    PLIC_BASE.offset((def::UART0_IRQ * 4) as usize).write(1);
    PLIC_BASE.offset((def::VIRTIO0_IRQ * 4) as usize).write(1);
}

pub fn init_hart() {
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

pub type IRQ = u32;

pub enum Source {
    Unknown(reg::Scause), // The source is not device
    Timer,                // Timer interrupt
    Device(IRQ),          // Device interrupt, the value is the IRQ number
}

/// Check if it's an external interrupt or software interrupt, and handle it.
pub fn dev_intr() -> Source {
    let scause = reg::scause.read();
    let hart = arch::cpuid();

    if scause.is_interrupt() {
        // Interrupt
        use reg::ScauseInterrupt;

        let irq = plic_claim(hart);
        match scause.interrupt() {
            ScauseInterrupt::SupervisorSoftwareInterrupt => {
                // Software interrupt from a machine-mode timer interrupt,
                // forwarded by timervec in kernelvec.S.
                if hart == 0 {
                    proc::timer_interrupt();
                }

                // Acknowledge the software interrupt by clearing
                // the SSIP bit in sip.
                unsafe { reg::sip.clear_ssip() };

                return Source::Timer;
            }
            ScauseInterrupt::SupervisorExternalInterrupt => {
                // This is a supervisor external interrupt, via PLIC.
                // irq indicates which device interrupted.

                #[allow(unused_variables)]
                match irq as usize {
                    def::UART0_IRQ => {
                        // TODO: uartintr();
                    }
                    def::VIRTIO0_IRQ => {
                        // TODO: virtio_disk_intr();
                    }
                    irq => {
                        println!("unexpected interrupt irq={}", irq);
                    }
                }
                // The PLIC allows each device to raise at most one
                // interrupt at a time; tell the PLIC the device is
                // now allowed to interrupt again.
                if irq != 0 {
                    plic_complete(hart, irq);
                }
                return Source::Device(irq);
            }
            other => {
                println!("unexpected interrupt scause={:x?}", other);
            }
        }
    } else {
        // Exception
    }
    return Source::Unknown(scause);
}
