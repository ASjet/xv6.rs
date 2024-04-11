mod sv39;
mod sv48;
mod sv57;

pub use sv39::Sv39;
pub use sv48::Sv48;
pub use sv57::Sv57;

use super::VirtAddr;
use crate::BitFlag;

#[derive(Clone, Copy, Debug)]
pub struct PageLevel {
    pub vpn: BitFlag,
    pub pte_ppn: BitFlag,
    pub pa_ppn: BitFlag,
    pub page_offset: BitFlag,
}

impl PageLevel {
    pub const fn new(vpn: BitFlag, pte_ppn: BitFlag, pa_ppn: BitFlag) -> Self {
        PageLevel {
            vpn,
            pte_ppn,
            pa_ppn,
            page_offset: BitFlag::new(pa_ppn.shift(), 0),
        }
    }
}

pub trait PagingSchema {
    /// The maximum virtual address of the schema
    fn max_va() -> VirtAddr;

    /// The mask for page address
    fn page_levels() -> &'static [PageLevel];
}
