use crate::{
    arch,
    proc::{InterruptLock, CPU},
};
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

#[derive(Debug)]
pub struct Mutex<T> {
    name: &'static str,     // Name of the lock for debugging
    data: UnsafeCell<T>,    // The data being protected
    locked: AtomicPtr<CPU>, // The CPU holding the lock
}

impl<T> Mutex<T> {
    pub const fn new(value: T, name: &'static str) -> Mutex<T> {
        Mutex {
            name: name,
            data: UnsafeCell::new(value),
            locked: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        unsafe {
            let cpu = CPU::this();
            let int_lock = cpu.push_off();

            assert!(!self.holding(), "acquire {}", self.name);

            loop {
                if self
                    .locked
                    .compare_exchange(
                        core::ptr::null_mut(),
                        cpu,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return MutexGuard {
                        mutex: self,
                        _int_lock: int_lock,
                    };
                }
                core::hint::spin_loop();
            }
        }
    }

    unsafe fn holding(&self) -> bool {
        self.locked.load(Ordering::Relaxed) == CPU::this()
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        &mut *self.data.get()
    }

    pub fn unlock(guard: MutexGuard<'_, T>) -> &'_ Mutex<T> {
        guard.mutex()
    }

    pub unsafe fn force_unlock(&self) {
        assert!(self.holding(), "force_unlock {}", self.name);
        self.locked.store(core::ptr::null_mut(), Ordering::Release);
        CPU::this().pop_off();
    }
}

unsafe impl<T> Sync for Mutex<T> {}
unsafe impl<T> Send for Mutex<T> {}

#[derive(Debug)]
pub struct MutexGuard<'a, T: 'a> {
    mutex: &'a Mutex<T>,
    _int_lock: InterruptLock,
}

impl<'a, T: 'a> MutexGuard<'a, T> {
    pub fn mutex(&self) -> &'a Mutex<T> {
        self.mutex
    }

    pub fn holding(&self) -> bool {
        assert!(!arch::is_intr_on(), "interrupt enabled");
        unsafe { self.mutex.holding() }
    }
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        assert!(self.holding(), "release {}", self.mutex.name);
        self.mutex.locked.store(ptr::null_mut(), Ordering::Release);
    }
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}
