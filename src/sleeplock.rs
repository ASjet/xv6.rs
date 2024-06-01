use crate::proc::{Proc, CPU};
use crate::spinlock::Mutex;
use core::cell::UnsafeCell;
use core::ptr::addr_of;

#[derive(Debug)]
struct SleepMutexSync {
    pid: Option<i32>,
    locked: bool,
}

#[derive(Debug)]
pub struct SleepMutex<T> {
    data: UnsafeCell<T>, // The data being protected
    locked: Mutex<SleepMutexSync>,
}

impl<T> SleepMutex<T> {
    pub const fn new(value: T, name: &'static str) -> SleepMutex<T> {
        SleepMutex {
            data: UnsafeCell::new(value),
            locked: Mutex::new(
                SleepMutexSync {
                    pid: None,
                    locked: false,
                },
                name,
            ),
        }
    }

    pub fn lock(&mut self) -> SleepMutexGuard<'_, T> {
        unsafe {
            let mut guard = self.locked.lock();
            while guard.locked {
                guard = CPU::this_proc_ref().sleep(addr_of!(self) as usize, guard);
            }
            guard.locked = true;
            guard.pid = CPU::this_proc_ref().pid();
        }
        SleepMutexGuard { mutex: self }
    }

    pub fn holding(&self) -> bool {
        let sync = self.locked.lock();
        sync.locked && sync.pid == unsafe { CPU::this_proc_ref().pid() }
    }

    pub unsafe fn get(&self) -> &T {
        &*self.data.get()
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        &mut *self.data.get()
    }

    pub fn unlock(guard: SleepMutexGuard<'_, T>) -> &'_ SleepMutex<T> {
        guard.mutex()
    }
}

unsafe impl<T> Sync for SleepMutex<T> {}
unsafe impl<T> Send for SleepMutex<T> {}

#[derive(Debug)]
pub struct SleepMutexGuard<'a, T> {
    mutex: &'a SleepMutex<T>,
}

impl<'a, T: 'a> SleepMutexGuard<'a, T> {
    pub fn mutex(&self) -> &'a SleepMutex<T> {
        self.mutex
    }

    pub fn holding(&self) -> bool {
        self.mutex.holding()
    }
}

impl<'a, T> core::ops::Drop for SleepMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.lock().locked = false;
        Proc::wake_up(addr_of!(*self.mutex) as usize);
    }
}

impl<'a, T> core::ops::Deref for SleepMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SleepMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}
