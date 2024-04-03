use super::{PageLevel, PagingSchema, VirtAddr, PAGE_OFFSET, PTE_FLAGS, VPN_WIDTH};
use crate::insn::Mask;

const VA_WIDTH: usize = 48;
const MAX_VA: VirtAddr = VirtAddr((1 << VA_WIDTH) - 1);

const PAGE_LEVELS: [PageLevel; 4] = [
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 0),
        Mask::new(17 + 9 + 9 + 9, PTE_FLAGS.width()),
        Mask::new(17 + 9 + 9 + 9, PAGE_OFFSET.width()),
    ),
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 1),
        Mask::new(17 + 9 + 9, PTE_FLAGS.width() + 9),
        Mask::new(17 + 9 + 9, PAGE_OFFSET.width() + 9),
    ),
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 2),
        Mask::new(17 + 9, PTE_FLAGS.width() + 9 + 9),
        Mask::new(17 + 9, PAGE_OFFSET.width() + 9 + 9),
    ),
    PageLevel::new(
        Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 3),
        Mask::new(17, PTE_FLAGS.width() + 9 + 9 + 9),
        Mask::new(17, PAGE_OFFSET.width() + 9 + 9 + 9),
    ),
];

pub struct Sv48;

impl PagingSchema for Sv48 {
    #[inline]
    fn max_va() -> VirtAddr {
        MAX_VA
    }

    #[inline]
    fn page_levels() -> &'static [super::PageLevel] {
        &PAGE_LEVELS
    }
}
