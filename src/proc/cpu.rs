use core::ptr::{addr_of, NonNull};

use super::{state, switch::Context, Proc};
use crate::{arch, spinlock};

extern "C" {
    fn switch(save: *const Context, load: *const Context);
}

pub static mut CPUS: [CPU; crate::NCPU] = [CPU::new(); crate::NCPU];
pub static mut TICKS: spinlock::Mutex<usize> = spinlock::Mutex::new(0, "time");

/// Per-CPU state
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CPU {
    proc: Option<NonNull<Proc>>, // Guarantee: `proc` is either `None` or `Some(valid_pointer)`
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
    pub fn this_proc() -> Option<NonNull<Proc>> {
        unsafe {
            let _guard = CPU::push_off();
            CPUS[arch::cpuid()].proc
        }
    }

    #[inline]
    /// Currently running process on this CPU
    /// Safety: Caller must ensure that there is a process running on this CPU
    pub unsafe fn this_proc_ref() -> &'static mut Proc {
        let _guard = CPU::push_off();
        CPUS[arch::cpuid()]
            .proc
            .expect("no process on this cpu")
            .as_mut()
    }

    #[inline]
    pub unsafe fn push_off() -> InterruptLock {
        let int_enabled = arch::is_intr_on();
        arch::intr_off();
        let cpu = &mut CPUS[arch::cpuid()];
        if cpu.noff == 0 {
            cpu.interrupt_enabled = int_enabled;
        }
        cpu.noff += 1;
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
    pub fn set_proc(&mut self, p: Option<NonNull<Proc>>) {
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

pub fn timer_interrupt() {
    unsafe {
        let mut ticks = TICKS.lock();
        *ticks += 1;
        state::Proc::wake_up(addr_of!(*ticks) as usize);
    }
}
