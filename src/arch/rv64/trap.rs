use super::def;
use crate::{arch, println};
use core::{arch::global_asm, panic};
use rv64::insn::{m, s, RegisterRO, RegisterRW};

static mut TIMER_SCRATCH: [[u64; 5]; crate::NCPU] = [[0; 5]; crate::NCPU];

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

pub unsafe fn init_hart() {
    extern "C" {
        fn kernel_vec();
    }
    s::stvec.write(kernel_vec as usize);
}

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

#[no_mangle]
extern "C" fn kernel_trap() {
    let sepc = s::sepc.read();
    let sstatus = s::sstatus.read();
    let scause = s::scause.read();

    if s::SSTATUS_SPP.get(sstatus) == 0 {
        panic!("kerneltrap: not from supervisor mode");
    }
    if arch::is_intr_on() {
        panic!("kerneltrap: interrupts enabled");
    }
    match dev_intr() {
        0 => {
            println!("scause {:x?}", scause);
            println!("sepc={:x?} stval={:x?}", s::sepc.read(), s::stval.read());
            // TODO: fix here with scause 1 or 4
            panic!("kernel_trap");
        }
        2 => {
            // give up the CPU if this is a timer interrupt.
            if arch::cpuid() != 0 {
                // TODO: make sure cpu state is RUNNING
                // TODO: yield();
            }
        }
        _ => {
            // Ignored
        }
    }
    unsafe {
        s::sepc.write(sepc);
        s::sstatus.write(sstatus);
    }
}

/// Check if it's an external interrupt or software interrupt, and handle it.
/// returns `2` if timer interrupt,
/// - `1` if other device,
/// - `0` if not recognized.
fn dev_intr() -> i32 {
    let scause = s::scause.read();
    let hart = arch::cpuid();

    if s::SCAUSE_INTERRUPT.get(scause) != 0 {
        // Interrupt
        use s::ScauseExceptInt;

        let except = s::SCAUSE_EXCEPT_INT.get(scause) & 0xff;
        let irq = arch::interrupt::plic_claim(hart);
        match ScauseExceptInt::try_from(except as u8).unwrap_or(ScauseExceptInt::Reserved) {
            ScauseExceptInt::SupervisorSoftwareInterrupt => {
                // Software interrupt from a machine-mode timer interrupt,
                // forwarded by timervec in kernelvec.S.
                if hart == 0 {
                    // TODO: clockintr();
                }

                // Acknowledge the software interrupt by clearing
                // the SSIP bit in sip.
                unsafe { s::sip.clear_mask(s::SIP_SSIP) };

                return 2;
            }
            ScauseExceptInt::SupervisorExternalInterrupt => {
                // This is a supervisor external interrupt, via PLIC.
                // irq indicates which device interrupted.

                match irq as u64 {
                    def::UART0_IRQ => {
                        // TODO: uartintr();
                        return 1;
                    }
                    def::VIRTIO0_IRQ => {
                        // TODO: virtio_disk_intr();
                        return 1;
                    }
                    irq => {
                        println!("unexpected interrupt irq={}", irq);
                    }
                }
                // The PLIC allows each device to raise at most one
                // interrupt at a time; tell the PLIC the device is
                // now allowed to interrupt again.
                if irq != 0 {
                    arch::interrupt::plic_complete(hart, irq);
                }
                return 1;
            }
            other => {
                println!("unexpected interrupt scause={:x?}", other);
            }
        }
    } else {
        // Non-interrupt
    }
    return 0;
}
