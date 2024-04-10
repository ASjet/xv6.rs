use crate::instruction;

instruction!(
    /// Flush TLB
    sfence_vma, "sfence.vma zero, zero"
);

instruction!(
    /// Return from S mode to U mode and jump to `sepc`
    unsafe sret, "sret", nomem, nostack
);
