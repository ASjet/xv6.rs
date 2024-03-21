use crate::{csr_reg_ro, mv_reg_rw};

csr_reg_ro!(
    /// Time counter
    time
);

csr_reg_ro!(
    /// Cycle counter
    cycle
);

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
