use super::{PagingSchema, VirtAddr, PAGE_OFFSET, PPN, PTE_FLAGS, VPN_WIDTH};
use crate::insn::Mask;

const PA_WIDTH: usize = 56;
const PAGE_ADDR: Mask = Mask::new(PA_WIDTH - PAGE_OFFSET.width(), PAGE_OFFSET.width());
const PTE_ADDR: Mask = Mask::new(PAGE_ADDR.width(), PTE_FLAGS.width());

const PPN_MASKS: [PPN; 3] = [
    PPN::new(
        Mask::new(9, PAGE_OFFSET.width()),
        Mask::new(9, PAGE_OFFSET.width()),
    ),
    PPN::new(
        Mask::new(9, PAGE_OFFSET.width() + 9),
        Mask::new(9, PAGE_OFFSET.width() + 9),
    ),
    PPN::new(
        Mask::new(26, PAGE_OFFSET.width() + 9 + 9),
        Mask::new(26, PAGE_OFFSET.width() + 9 + 9),
    ),
];

const VA_WIDTH: usize = 39;
const MAX_VA: VirtAddr = VirtAddr((1 << VA_WIDTH) - 1);

const VPN_MASKS: [Mask; 3] = [
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 0),
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 1),
    Mask::new(VPN_WIDTH, PAGE_OFFSET.width() + VPN_WIDTH * 2),
];

pub struct Sv39;

impl PagingSchema for Sv39 {
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
    fn ppn() -> &'static [PPN] {
        &PPN_MASKS
    }

    #[inline]
    fn va_vpn() -> &'static [Mask] {
        &VPN_MASKS
    }
}
