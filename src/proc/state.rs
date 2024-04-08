use super::{cpu, switch, CPU};
use crate::{
    arch::{
        self,
        def::{self, PG_SIZE},
    },
    mem::alloc::ALLOCATOR,
    spinlock::Mutex,
};
use core::{mem::size_of, ptr::addr_of_mut};

pub static GLOBAL_LOCK: Mutex<()> = Mutex::new((), "global_proc_lock");

pub type Pid = i32;
static NEXT_PID: Mutex<Pid> = Mutex::new(1, "next_pid");

/// Allocate a globally unique PID
pub fn alloc_pid() -> Pid {
    let mut next_pid = NEXT_PID.lock();
    let pid = *next_pid;
    *next_pid += 1;
    pid
}

static mut _PROC_MEM: [usize; size_of::<[Proc; crate::NPROC]>() / size_of::<usize>()] =
    [0; size_of::<[Proc; crate::NPROC]>() / size_of::<usize>()];
pub static mut PROCS: *mut [Proc; crate::NPROC] = core::ptr::null_mut();

pub fn kstack_addrs() -> [usize; crate::NPROC] {
    core::array::from_fn(arch::def::kstack)
}

pub fn init() {
    unsafe {
        PROCS = addr_of_mut!(_PROC_MEM) as *mut [Proc; crate::NPROC];
        (*PROCS).iter_mut().enumerate().for_each(|(i, proc)| {
            *proc = Proc::new(def::kstack(i));
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Unused,
    Used,
    Sleeping,
    Runnable,
    Running,
    Zombie,
}

#[derive(Debug)]
struct _ProcSync {
    state: State,
    chan: *mut (),
    killed: bool,
    xstate: i32,
    pid: Option<Pid>,
}

#[derive(Debug)]
pub struct Proc {
    /// States that need to sync to all threads
    sync: Mutex<_ProcSync>,
    /// NOTE: `GLOBAL_LOCK` must be held when using these
    parent: *mut Proc,

    // these are private to the process, so no synchronization is needed
    /// Process name
    name: [u8; 16],
    /// Virtual address of kernel stack
    kstack: usize,
    /// Size of process memory in bytes
    size: usize,
    /// User page table
    pagetable: *mut arch::vm::PageTable,
    /// Data for trampoline
    trapframe: *mut arch::trampoline::TrapFrame,
    /// swtch() here to run process
    context: switch::Context,
    // TODO: array[NOFILE] of opened file descriptors
    // TODO: *inode for cwd
}

impl Proc {
    const fn new(kstack: usize) -> Proc {
        Proc {
            sync: Mutex::new(
                _ProcSync {
                    state: State::Unused,
                    chan: core::ptr::null_mut(),
                    killed: false,
                    xstate: 0,
                    pid: None,
                },
                "proc_sync",
            ),
            parent: core::ptr::null_mut(),
            name: [0; 16],
            kstack,
            size: 0,
            pagetable: core::ptr::null_mut(),
            trapframe: core::ptr::null_mut(),
            context: switch::Context::new(),
        }
    }

    pub fn state(&self) -> State {
        self.sync.lock().state
    }

    pub fn cas_state(&self, old: State, new: State) -> bool {
        let mut sync = self.sync.lock();
        if sync.state == old {
            sync.state = new;
            true
        } else {
            false
        }
    }

    /// Switch to scheduler.  Must hold only p->lock
    /// and have changed proc->state. Saves and restores
    /// intena because intena is a property of this
    /// kernel thread, not this CPU. It should
    /// be proc->intena and proc->noff, but that would
    /// break in the few places where a lock is held but
    /// there's no process.
    pub unsafe fn sched(&self) {
        let c = CPU::this();
        assert!(self.sync.holding(), "sched proc not locked");
        assert!((*c).get_noff() == 1, "sched cpu locks");
        assert_ne!(self.sync.get().state, State::Running, "sched proc running");
        assert!(!arch::is_intr_on(), "sched interruptible");

        let int_enable = (*c).get_interrupt_enabled();

        unsafe {
            (*c).switch_back(&self.context);
            (*CPU::this_mut()).set_interrupt_enabled(int_enable);
        }
    }

    /// Give up the CPU for one scheduling round.
    pub fn r#yield(&mut self) {
        let mut sync = self.sync.lock();
        sync.state = State::Runnable;
        unsafe { self.sched() };
    }

    /// Look in the process table for an UNUSED proc.
    /// If found, initialize state required to run in the kernel,
    /// and return with p->lock held.(FIXME: Is holding lock necessary?)
    /// If there are no free procs, or a memory allocation fails, return 0.
    pub fn alloc() -> Option<*mut Proc> {
        let p = unsafe {
            (*PROCS)
                .iter_mut()
                .filter(|p| p.cas_state(State::Unused, State::Used))
                .next()
        }?;

        p.sync.lock().pid = Some(alloc_pid());

        p.trapframe = unsafe {
            ALLOCATOR.kalloc().or_else(|| {
                p.free();
                None
            })
        }?
        .as_mut_ptr::<arch::trampoline::TrapFrame>();

        p.pagetable = unsafe {
            ALLOCATOR.kalloc().or_else(|| {
                p.free();
                None
            })
        }?
        .as_mut_ptr::<arch::vm::PageTable>();

        p.context.setup(fork_ret as usize, p.kstack + PG_SIZE);

        Some(p)
    }

    /// free a proc structure and the data hanging from it,
    /// including user pages.
    /// p->lock must be held.(FIXME: Is holding lock necessary?)
    pub fn free(&mut self) {
        if !self.trapframe.is_null() {
            unsafe {
                ALLOCATOR.kfree(self.trapframe);
            }
            self.trapframe = core::ptr::null_mut();
        }
        if !self.pagetable.is_null() {
            unsafe {
                ALLOCATOR.kfree(self.pagetable);
            }
            self.pagetable = core::ptr::null_mut();
        }
        self.size = 0;
        self.parent = core::ptr::null_mut();
        self.name = [0; 16];

        let mut sync = self.sync.lock();
        sync.state = State::Unused;
        sync.pid = None;
        sync.chan = core::ptr::null_mut();
        sync.killed = false;
        sync.xstate = 0;
    }
}

/// Per-CPU process scheduler.
/// Each CPU calls scheduler() after setting itself up.
/// Scheduler never returns.  It loops, doing:
///  - choose a process to run.
///  - swtch to start running that process.
///  - eventually that process transfers control
///    via swtch back to the scheduler.
pub fn scheduler() -> ! {
    let c = unsafe { cpu::CPU::this_mut() };
    loop {
        // Avoid deadlock by ensuring that devices can interrupt.
        arch::intr_on();

        unsafe {
            (*PROCS)
                .iter_mut()
                .filter(|p| p.cas_state(State::Runnable, State::Running))
                .for_each(|run| {
                    // Switch to chosen process
                    (*c).set_proc(Some(run));

                    // It is the process's job to release its lock and then
                    // reacquire it before jumping back to us.
                    let _guard = run.sync.lock();
                    (*c).switch_to(&run.context);

                    // Process is done running for now.
                    // It should have changed its p->state before coming back.
                    (*c).set_proc(None);
                });
        }

        // No process to run, wait for an interrupt.
        core::hint::spin_loop();
    }
}

fn fork_ret() {
    todo!()
}
