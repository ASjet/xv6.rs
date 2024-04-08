use super::{switch::Context, Proc};
use crate::arch;

extern "C" {
    fn switch(save: *const Context, load: *const Context);
}

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

    /// Return this CPU's cpu struct.
    /// Interrupts must be disabled.
    #[inline]
    pub unsafe fn this_mut() -> *mut CPU {
        &mut CPUS[arch::cpuid()]
    }

    /// Return this CPU's cpu struct.
    /// Interrupts must be disabled.
    #[inline]
    pub unsafe fn this() -> *const CPU {
        &CPUS[arch::cpuid()]
    }

    #[inline]
    /// Currently running process on this CPU
    pub unsafe fn this_proc() -> Option<*mut Proc> {
        let _guard = CPU::push_off();
        (*CPU::this_mut()).proc
    }

    #[inline]
    pub unsafe fn push_off() -> InterruptLock {
        let int_enabled = arch::is_intr_on();
        arch::intr_off();
        let c = CPU::this_mut();
        if (*c).noff == 0 {
            (*c).interrupt_enabled = int_enabled;
        }
        (*c).noff += 1;
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
    pub unsafe fn switch_to(&self, p: *const Context) {
        switch(&self.context, p);
    }

    /// Switch back to origin context, return to `switch_to`
    #[inline]
    pub unsafe fn switch_back(&self, p: *const Context) {
        switch(p, &self.context);
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
            (*CPU::this_mut()).pop_off();
        }
    }
}
