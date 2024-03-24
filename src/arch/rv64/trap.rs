use super::def;
use core::arch::global_asm;
use rv64::insn::{m, RegisterRW};

static mut TIMER_SCRATCH: [[u64; 5]; crate::NCPU] = [[0; 5]; crate::NCPU];

global_asm!(
    "
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
        let mtimecmp_ptr = def::clint_mtimecmp(hart_id as u64) as *mut u64;
        *(mtimecmp_ptr) = *(def::CLINT_MTIME as *const u64) + interval;

        let scratch = &mut TIMER_SCRATCH[hart_id];
        scratch[3] = mtimecmp_ptr as u64;
        scratch[4] = interval;
        m::mscratch.write(scratch.as_ptr() as usize);

        m::mtvec.write(timer_vec as usize);

        // Enable machine-mode interrupts.
        m::mstatus.set_mask(m::MSTATUS_MIE);

        // Enable machine-mode timer interrupts.
        m::mie.set_mask(m::MIE_MTIE);
    }
}
