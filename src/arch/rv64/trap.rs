use super::def::{TRAMPOLINE, TRAP_FRAME};
use super::{def, interrupt, intr_off, vm};
use crate::proc::{State, CPU};
use crate::{arch, println};
use core::{arch::global_asm, panic};
use rv64::reg::{self, RegisterRO, RegisterRW};
use rv64::BitFlagOps;

static mut TIMER_SCRATCH: [[u64; 5]; crate::NCPU] = [[0; 5]; crate::NCPU];

pub unsafe fn init_hart() {
    extern "C" {
        fn kernel_vec();
    }
    reg::stvec.write((kernel_vec as usize).into());
}

global_asm!(
    "
        #
        # machine-mode timer interrupt.
        #
.globl timer_vec
.align 4
timer_vec:
        # start.c has set up the memory that mscratch points to:
        # scratch[0,8,16] : register save area.
        # scratch[24] : address of CLINT's MTIMECMP register.
        # scratch[32] : desired interval between interrupts.

        csrrw a0, mscratch, a0
        sd a1, 0(a0)
        sd a2, 8(a0)
        sd a3, 16(a0)

        # schedule the next timer interrupt
        # by adding interval to mtimecmp.
        ld a1, 24(a0) # CLINT_MTIMECMP(hart)
        ld a2, 32(a0) # interval
        ld a3, 0(a1)
        add a3, a3, a2
        sd a3, 0(a1)

        # raise a supervisor software interrupt.
        li a1, 2
        csrw sip, a1

        ld a3, 16(a0)
        ld a2, 8(a0)
        ld a1, 0(a0)
        csrrw a0, mscratch, a0

        mret
"
);

pub unsafe fn init_timer_interrupt(hart_id: usize) {
    extern "C" {
        fn timer_vec();
    }

    unsafe {
        let interval = 1000000; // cycles; about 1/10th second in qemu.

        // ask the CLINT for a timer interrupt.
        let mtimecmp_ptr = def::clint_mtimecmp(hart_id) as *mut u64;
        *(mtimecmp_ptr) = *(def::CLINT_MTIME as *const u64) + interval;

        let scratch = &mut TIMER_SCRATCH[hart_id];
        scratch[3] = mtimecmp_ptr as u64;
        scratch[4] = interval;
        reg::mscratch.write(scratch.as_ptr() as usize);

        reg::mtvec.write((timer_vec as usize).into());

        // Enable machine-mode interrupts.
        reg::mstatus.set_mie();

        // Enable machine-mode timer interrupts.
        reg::mie.set_mtie();
    }
}

global_asm!(
    "
    .globl kernel_trap
    .globl kernel_vec
    .align 4
    kernel_vec:
            // make room to save registers.
            addi sp, sp, -256

            // save the registers.
            sd ra, 0(sp)
            sd sp, 8(sp)
            sd gp, 16(sp)
            sd tp, 24(sp)
            sd t0, 32(sp)
            sd t1, 40(sp)
            sd t2, 48(sp)
            sd s0, 56(sp)
            sd s1, 64(sp)
            sd a0, 72(sp)
            sd a1, 80(sp)
            sd a2, 88(sp)
            sd a3, 96(sp)
            sd a4, 104(sp)
            sd a5, 112(sp)
            sd a6, 120(sp)
            sd a7, 128(sp)
            sd s2, 136(sp)
            sd s3, 144(sp)
            sd s4, 152(sp)
            sd s5, 160(sp)
            sd s6, 168(sp)
            sd s7, 176(sp)
            sd s8, 184(sp)
            sd s9, 192(sp)
            sd s10, 200(sp)
            sd s11, 208(sp)
            sd t3, 216(sp)
            sd t4, 224(sp)
            sd t5, 232(sp)
            sd t6, 240(sp)

            call kernel_trap

            // restore registers.
            ld ra, 0(sp)
            ld sp, 8(sp)
            ld gp, 16(sp)
            // not this, in case we moved CPUs: ld tp, 24(sp)
            ld t0, 32(sp)
            ld t1, 40(sp)
            ld t2, 48(sp)
            ld s0, 56(sp)
            ld s1, 64(sp)
            ld a0, 72(sp)
            ld a1, 80(sp)
            ld a2, 88(sp)
            ld a3, 96(sp)
            ld a4, 104(sp)
            ld a5, 112(sp)
            ld a6, 120(sp)
            ld a7, 128(sp)
            ld s2, 136(sp)
            ld s3, 144(sp)
            ld s4, 152(sp)
            ld s5, 160(sp)
            ld s6, 168(sp)
            ld s7, 176(sp)
            ld s8, 184(sp)
            ld s9, 192(sp)
            ld s10, 200(sp)
            ld s11, 208(sp)
            ld t3, 216(sp)
            ld t4, 224(sp)
            ld t5, 232(sp)
            ld t6, 240(sp)

            addi sp, sp, 256

            // return to whatever we were doing in the kernel.
            sret

"
);

#[no_mangle]
extern "C" fn kernel_trap() {
    let sepc_v = reg::sepc.read();
    let sstatus_v = reg::sstatus.read();

    assert!(
        sstatus_v.spp() == rv64::PrivilegeLevel::S,
        "kernel trap: not from supervisor mode"
    );
    assert!(!arch::is_intr_on(), "kernel_trap: interrupts enabled");

    use interrupt::Source;
    match interrupt::dev_intr() {
        Source::Unknown(scause) => {
            panic!(
                "kernel_trap: scause: {:x?}, sep: {:x?}, stval: {:x?}",
                scause,
                reg::sepc.read(),
                reg::stval.read()
            );
        }
        Source::Timer => {
            // give up the CPU if this is a timer interrupt.
            if arch::cpuid() != 0 {
                unsafe {
                    if let Some(mut p) = CPU::this_proc() {
                        let p = p.as_mut();
                        if p.state() == State::Running {
                            p.r#yield();
                        }
                    }
                }
            }
        }
        Source::Device(_) => {
            // Ignored
        }
    }
    unsafe {
        reg::sepc.write(sepc_v);
        reg::sstatus.write(sstatus_v);
    }
}

/// handle an interrupt, exception, or system call from user space.
/// called from trampoline.S
extern "C" fn user_trap() {
    extern "C" {
        fn kernelvec();
    }

    assert_eq!(
        reg::sstatus.read().spp(),
        rv64::PrivilegeLevel::U,
        "user_trap: not from user mode"
    );

    let mut which_dev = interrupt::Source::Unknown(0.into());

    unsafe {
        // send interrupts and exceptions to kerneltrap(),
        // since we're now in the kernel.
        reg::stvec.write((kernelvec as usize).into());

        let p = CPU::this_proc_ref();
        let trapframe = p.trapframe().unwrap_unchecked().as_mut();
        trapframe.epc = reg::sepc.read();

        match reg::scause.read().into() {
            // system call
            8 => {
                if p.killed() {
                    p.exit(-1);
                }

                // sepc points to the ecall instruction,
                // but we want to return to the next instruction.
                trapframe.epc += 4;

                // an interrupt will change sstatus &c registers,
                // so don't enable until done with those registers.
                arch::intr_on();

                // TODO: syscall()
            }
            scause_v => {
                which_dev = interrupt::dev_intr();
                if let interrupt::Source::Unknown(_) = which_dev {
                    println!(
                        "user_trap: unexpected scause={:x?}, pid={}",
                        scause_v,
                        p.pid().unwrap()
                    );
                    println!(
                        "           spec={:x?}, stval={:x?}",
                        reg::sepc.read(),
                        reg::stval.read(),
                    );
                    p.set_killed(true);
                }
            }
        }

        if p.killed() {
            p.exit(-1);
        }

        // give up the CPU if this is a timer interrupt.
        if let interrupt::Source::Timer = which_dev {
            p.r#yield();
        }
    };
    user_trap_ret();
}

/// Return to user space
#[no_mangle]
pub extern "C" fn user_trap_ret() {
    // we're about to switch the destination of traps from
    // kerneltrap() to usertrap(), so turn off interrupts until
    // we're back in user space, where usertrap() is correct.
    intr_off();

    unsafe {
        // send syscalls, interrupts, and exceptions to trampoline.S
        reg::stvec.write((TRAMPOLINE + (vm::uservec() - vm::trampoline())).into());

        // set up trapframe values that uservec will need when
        // the process next re-enters the kernel.
        let p = CPU::this_proc_ref();
        let trapframe = p.trapframe().unwrap_unchecked().as_mut();
        trapframe.kernel_satp = reg::satp.read(); // kernel page table
        trapframe.kernel_sp = p.kstack(); // process's kernel stack
        trapframe.kernel_trap = user_trap as usize;
        trapframe.kernel_hartid = arch::cpuid(); // hartid for cpuid()

        // set up the registers that trampoline.S's sret will use
        // to get to user space.

        // set S Previous Privilege mode to User.
        reg::sstatus.write(
            reg::sstatus
                .read()
                .clear_mask(&reg::sstatus::SPP) // clear SPP to 0 for user mode
                .set_mask(&reg::sstatus::SPIE) // enable interrupts in user mode
                .into(),
        );

        // set S Exception Program Counter to the saved user pc.
        reg::sepc.write(trapframe.epc);

        // tell trampoline.S the user page table to switch to.
        let satp_v = reg::satp.make(reg::SatpMode::Sv39, 0, p.pagetable().into());

        // jump to trampoline.S at the top of memory, which
        // switches to the user page table, restores user registers,
        // and switches to user mode with sret.
        let func: extern "C" fn(usize, usize) =
            core::mem::transmute(TRAMPOLINE + (vm::userret() - vm::trampoline()));
        func(TRAP_FRAME, satp_v.into());
    }
}
