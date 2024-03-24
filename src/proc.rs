use crate::arch;

/// Saved registers for kernel context switches.
#[derive(Clone, Copy)]
#[repr(C)]
struct Context {
    ra: u64,
    sp: u64,

    // callee-saved
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
    s4: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
    s9: u64,
    s10: u64,
    s11: u64,
}

impl Context {
    const fn new() -> Context {
        return Context {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        };
    }
}

pub static mut CPUS: [CPU; crate::NCPU] = [CPU::new(); crate::NCPU];

/// Per-CPU state
#[derive(Clone, Copy)]
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

    pub unsafe fn this() -> &'static mut CPU {
        &mut CPUS[arch::cpuid()]
    }

    pub unsafe fn lock(&mut self) -> InterruptLock {
        self.push_off();
        InterruptLock
    }

    pub unsafe fn push_off(&mut self) {
        let int_enabled = arch::is_intr_on();
        arch::intr_off();
        if self.noff == 0 {
            self.interrupt_enabled = int_enabled;
        }
        self.noff += 1;
    }

    pub unsafe fn pop_off(&mut self) {
        assert!(!arch::is_intr_on(), "pop_off - interruptible");
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
            CPU::this().pop_off();
        }
    }
}
