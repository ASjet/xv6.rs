use crate::{csr_reg_ro, instruction, mv_reg_rw};

instruction!(wfi, "wfi", nomem, nostack);

mv_reg_rw!(
    /// Return address
    ra
);

mv_reg_rw!(
    /// Stack pointer
    sp
);

mv_reg_rw!(
    /// Global pointer
    gp
);

mv_reg_rw!(
    /// Thread pointer, which should holds this core's hartid
    tp
);

mv_reg_rw!(
    /// Frame pointer
    fp
);

/*            Unprivileged Floating-Point CSRs            */

mv_reg_rw!(
    /// Floating-Point Accrued Exceptions
    fflags
);

mv_reg_rw!(
    /// Floating-Point Dynamic Rounding Mode
    frm
);

mv_reg_rw!(
    /// Floating-Point Control and Status Register(frm+fflags)
    fcsr
);

/*            Unprivileged Counter/Timers            */

csr_reg_ro!(
    /// Cycle counter for RDCYCLE instruction
    cycle
);

csr_reg_ro!(
    /// Time counter for RDTIME instruction
    time
);

csr_reg_ro!(
    /// Instructions-retired counter for RDINSTRET instruction
    instret
);
