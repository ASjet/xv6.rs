use super::{PageLevel, PagingSchema, VirtAddr, PAGE_OFFSET, PTE_FLAGS, VPN_WIDTH};
use crate::insn::Mask;

const VA_WIDTH: usize = 39;
const MAX_VA: VirtAddr = VirtAddr((1 << VA_WIDTH) - 1);

const PAGE_LEVELS: [PageLevel; 3] = [
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 0),
        Mask::new(26 + 9 + 9, PTE_FLAGS.width()),
        Mask::new(26 + 9 + 9, PAGE_OFFSET.width()),
    ),
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 1),
        Mask::new(26 + 9, PTE_FLAGS.width() + 9),
        Mask::new(26 + 9, PAGE_OFFSET.width() + 9),
    ),
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 2),
        Mask::new(26, PTE_FLAGS.width() + 9 + 9),
        Mask::new(26, PAGE_OFFSET.width() + 9 + 9),
    ),
];

pub struct Sv39;

impl PagingSchema for Sv39 {
    #[inline]
    fn max_va() -> VirtAddr {
        MAX_VA
    }

    #[inline]
    fn page_levels() -> &'static [super::PageLevel] {
        &PAGE_LEVELS
    }
}
