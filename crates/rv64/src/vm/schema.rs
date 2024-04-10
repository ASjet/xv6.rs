mod sv39;
mod sv48;
mod sv57;

pub use sv39::Sv39;
pub use sv48::Sv48;
pub use sv57::Sv57;

use super::VirtAddr;
use crate::Mask;

#[derive(Clone, Copy, Debug)]
pub struct PageLevel {
    pub vpn: Mask,
    pub pte_ppn: Mask,
    pub pa_ppn: Mask,
    pub page_offset: Mask,
}

impl PageLevel {
    pub const fn new(vpn: Mask, pte_ppn: Mask, pa_ppn: Mask) -> Self {
        PageLevel {
            vpn,
            pte_ppn,
            pa_ppn,
            page_offset: Mask::new(pa_ppn.shift(), 0),
        }
    }
}

pub trait PagingSchema {
    /// The maximum virtual address of the schema
    fn max_va() -> VirtAddr;

    /// The mask for page address
    fn page_levels() -> &'static [PageLevel];
}
