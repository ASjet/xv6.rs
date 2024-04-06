use super::cpu;
use crate::{
    arch::{self, def},
    spinlock::Mutex,
};
use core::mem::size_of;

pub type Pid = i32;
pub static NEXT_PID: Mutex<Pid> = Mutex::new(1, "next_pid");
pub static GLOBAL_LOCK: Mutex<()> = Mutex::new((), "global_proc_lock");

#[derive(Debug, Clone, Copy)]
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
    pid: Pid,
}

static mut _PROC_MEM: [u8; size_of::<[Proc; crate::NPROC]>()] =
    [0; size_of::<[Proc; crate::NPROC]>()];
pub static mut PROCS: *mut [Proc; crate::NPROC] = core::ptr::null_mut();

pub fn init() {
    unsafe {
        PROCS = _PROC_MEM.as_mut_ptr() as *mut [Proc; crate::NPROC];
        (*PROCS).iter_mut().enumerate().for_each(|(i, proc)| {
            *proc = Proc::new(def::kstack(i));
        });
    }
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
    context: cpu::Context,
    // TODO: array[NOFILE] of opened file descriptors
    // TODO: *inode for cwd
}

impl Proc {
    pub const fn new(kstack: usize) -> Proc {
        Proc {
            sync: Mutex::new(
                _ProcSync {
                    state: State::Unused,
                    chan: core::ptr::null_mut(),
                    killed: false,
                    xstate: 0,
                    pid: 0,
                },
                "proc_sync",
            ),
            parent: core::ptr::null_mut(),
            name: [0; 16],
            kstack,
            size: 0,
            pagetable: core::ptr::null_mut(),
            trapframe: core::ptr::null_mut(),
            context: cpu::Context::new(),
        }
    }
}

pub fn kstack_addrs() -> [usize; crate::NPROC] {
    core::array::from_fn(arch::def::kstack)
}
