use super::RegisterRW;
use crate::mv_reg_rw;
use core::arch::asm;

mv_reg_rw!(
    /// Stack pointer
    sp
);

mv_reg_rw!(
    /// Thread pointer, which should holds this core's hartid
    tp
);

mv_reg_rw!(
    /// Return address
    ra
);
