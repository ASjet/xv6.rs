use super::switch::Context;
use crate::arch;

pub static mut CPUS: [CPU; crate::NCPU] = [CPU::new(); crate::NCPU];

/// Per-CPU state
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CPU {
    _context: Context,
    noff: i32,
    interrupt_enabled: bool,
}

impl CPU {
    pub const fn new() -> CPU {
        CPU {
            _context: Context::new(),
            noff: 0,
            interrupt_enabled: false,
        }
    }

    pub unsafe fn this_mut() -> &'static mut CPU {
        &mut CPUS[arch::cpuid()]
    }

    pub unsafe fn this() -> &'static CPU {
        &CPUS[arch::cpuid()]
    }

    pub unsafe fn push_off(&mut self) -> InterruptLock {
        let int_enabled = arch::is_intr_on();
        arch::intr_off();
        if self.noff == 0 {
            self.interrupt_enabled = int_enabled;
        }
        self.noff += 1;
        InterruptLock
    }

    pub unsafe fn pop_off(&mut self) {
        assert!(!arch::is_intr_on(), "pop_off - interruptible");
        // FIXME: panic here
        assert!(self.noff >= 1, "pop_off");
        self.noff -= 1;
        if self.noff == 0 && self.interrupt_enabled {
            arch::intr_on();
        }
    }
}

#[derive(Debug)]
pub struct InterruptLock;

impl Drop for InterruptLock {
    fn drop(&mut self) {
        unsafe {
            CPU::this_mut().pop_off();
        }
    }
}
