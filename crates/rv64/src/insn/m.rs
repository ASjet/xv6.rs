use crate::instruction;

instruction!(
    /// Return from M mode to S mode and jump to `mepc`
    unsafe mret, "mret", nomem, nostack
);
