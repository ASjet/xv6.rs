use super::{switch::Context, Proc};
use crate::arch;

pub static mut CPUS: [CPU; crate::NCPU] = [CPU::new(); crate::NCPU];

/// Per-CPU state
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CPU {
    proc: Option<*mut Proc>,
    context: Context,
    noff: i32,
    interrupt_enabled: bool,
}

impl CPU {
    pub const fn new() -> CPU {
        CPU {
            proc: None,
            context: Context::new(),
            noff: 0,
            interrupt_enabled: false,
        }
    }

    #[inline]
    pub unsafe fn this_mut() -> &'static mut CPU {
        &mut CPUS[arch::cpuid()]
    }

    #[inline]
    pub unsafe fn this() -> &'static CPU {
        &CPUS[arch::cpuid()]
    }

    #[inline]
    /// Currently running process on this CPU
    pub unsafe fn proc(&mut self) -> Option<*mut Proc> {
        let _guard = self.push_off();
        self.proc
    }

    #[inline]
    pub unsafe fn push_off(&mut self) -> InterruptLock {
        let int_enabled = arch::is_intr_on();
        arch::intr_off();
        if self.noff == 0 {
            self.interrupt_enabled = int_enabled;
        }
        self.noff += 1;
        InterruptLock
    }

    #[inline]
    pub unsafe fn pop_off(&mut self) {
        assert!(!arch::is_intr_on(), "pop_off - interruptible");
        assert!(self.noff >= 1, "pop_off");
        self.noff -= 1;
        if self.noff == 0 && self.interrupt_enabled {
            arch::intr_on();
        }
    }

    /// Switch to another context, return to `switch_back`
    #[inline]
    pub unsafe fn switch_to(&self, p: &Context) {
        self.context.switch(p);
    }

    /// Switch back to origin context, return to `switch_to`
    #[inline]
    pub unsafe fn switch_back(&self, p: &Context) {
        p.switch(&self.context);
    }

    #[inline]
    pub fn set_proc(&mut self, p: Option<*mut Proc>) {
        self.proc = p;
    }

    #[inline]
    pub fn get_noff(&self) -> i32 {
        self.noff
    }

    #[inline]
    pub fn get_interrupt_enabled(&self) -> bool {
        self.interrupt_enabled
    }

    #[inline]
    pub fn set_interrupt_enabled(&mut self, enabled: bool) {
        self.interrupt_enabled = enabled;
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
