use super::{PageLevel, PagingSchema};
use crate::vm::{VirtAddr, PAGE_OFFSET, PTE, VPN_WIDTH};
use crate::BitFlag;

const VA_WIDTH: usize = 57;
const MAX_VA: VirtAddr = VirtAddr((1 << VA_WIDTH) - 1);

const PAGE_LEVELS: [PageLevel; 5] = [
    PageLevel::new(
        BitFlag::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 0),
        BitFlag::new(8 + 9 + 9 + 9 + 9, PTE::FLAGS.width()),
        BitFlag::new(8 + 9 + 9 + 9 + 9, PAGE_OFFSET.width()),
    ),
    PageLevel::new(
        BitFlag::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 1),
        BitFlag::new(8 + 9 + 9 + 9, PTE::FLAGS.width() + 9),
        BitFlag::new(8 + 9 + 9 + 9, PAGE_OFFSET.width() + 9),
    ),
    PageLevel::new(
        BitFlag::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 2),
        BitFlag::new(8 + 9 + 9, PTE::FLAGS.width() + 9 + 9),
        BitFlag::new(8 + 9 + 9, PAGE_OFFSET.width() + 9 + 9),
    ),
    PageLevel::new(
        BitFlag::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 3),
        BitFlag::new(8 + 9, PTE::FLAGS.width() + 9 + 9 + 9),
        BitFlag::new(8 + 9, PAGE_OFFSET.width() + 9 + 9 + 9),
    ),
    PageLevel::new(
        BitFlag::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 4),
        BitFlag::new(8, PTE::FLAGS.width() + 9 + 9 + 9 + 9),
        BitFlag::new(8, PAGE_OFFSET.width() + 9 + 9 + 9 + 9),
    ),
];

pub struct Sv57;

unsafe impl PagingSchema for Sv57 {
    #[inline]
    fn max_va() -> VirtAddr {
        MAX_VA
    }

    #[inline]
    fn page_levels() -> &'static [PageLevel] {
        &PAGE_LEVELS
    }
}
