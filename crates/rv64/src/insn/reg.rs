use crate::mv_reg_rw;

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
