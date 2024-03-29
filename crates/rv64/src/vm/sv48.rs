use super::{PagingSchema, VirtAddr, PAGE_OFFSET, PTE_FLAGS, VPN_WIDTH};
use crate::insn::Mask;

const VA_WIDTH: usize = 48;
const PA_WIDTH: usize = 56;

const VPN_MASKS: [Mask; 4] = [
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 3),
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 2),
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 1),
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 0),
];

const PPN_MASKS: [Mask; 4] = [
    Mask::new(17, PAGE_OFFSET.width() + 9 + 9 + 9),
    Mask::new(9, PAGE_OFFSET.width() + 9 + 9),
    Mask::new(9, PAGE_OFFSET.width() + 9),
    Mask::new(9, PAGE_OFFSET.width()),
];

const PAGE_ADDR: Mask = Mask::new(PA_WIDTH - PAGE_OFFSET.width(), PAGE_OFFSET.width());
const PTE_ADDR: Mask = Mask::new(PA_WIDTH - PAGE_OFFSET.width(), PTE_FLAGS.width());
const MAX_VA: VirtAddr = VirtAddr((1 << VA_WIDTH) - 1);

pub struct Addr;

impl PagingSchema for Addr {
    #[inline]
    fn max_va() -> VirtAddr {
        MAX_VA
    }

    #[inline]
    fn page_addr() -> &'static Mask {
        &PAGE_ADDR
    }

    #[inline]
    fn pte_addr() -> &'static Mask {
        &PTE_ADDR
    }

    #[inline]
    fn pte_ppn() -> &'static [Mask] {
        &PPN_MASKS
    }

    #[inline]
    fn va_vpn() -> &'static [Mask] {
        &VPN_MASKS
    }
}
