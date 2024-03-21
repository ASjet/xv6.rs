use super::Register;
use crate::mv_reg;
use core::arch::asm;

mv_reg!(
    /// Stack pointer
    sp
);

mv_reg!(
    /// Thread pointer, which should holds this core's hartid
    tp
);

mv_reg!(
    /// Return address
    ra
);
