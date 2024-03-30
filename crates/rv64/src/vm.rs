use crate::insn::Mask;
use core::{
    marker::PhantomData,
    mem::size_of,
    ops::{Add, Index, Sub},
};

mod sv39;
mod sv48;
mod sv57;

pub use sv39::Sv39;
pub use sv48::Sv48;
pub use sv57::Sv57;

/// The offset inside a page frame
pub const PAGE_OFFSET: Mask = Mask::new(12, 0);
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET.width();
pub const VPN_WIDTH: usize = PAGE_OFFSET.width() - 3; // sizeof::<usize>() == 2^3

pub const PTE_FLAGS: Mask = Mask::new(10, 0);
pub const PTE_V: Mask = Mask::new(1, 0);
pub const PTE_R: Mask = Mask::new(1, 1);
pub const PTE_W: Mask = Mask::new(1, 2);
pub const PTE_X: Mask = Mask::new(1, 3);
pub const PTE_XWR: Mask = Mask::new(3, 1);
pub const PTE_U: Mask = Mask::new(1, 4);
pub const PTE_G: Mask = Mask::new(1, 5);
pub const PTE_A: Mask = Mask::new(1, 6);
pub const PTE_D: Mask = Mask::new(1, 7);
pub const PTE_RSW: Mask = Mask::new(2, 8);

pub const PTE_PPN: Mask = Mask::new(44, 10);

/// Reserved for future standard use and, until their use is defined by some
/// standard extension, must be zeroed by software for forward compatibility.
/// If any of these bits are set, a page-fault exception is raised.
pub const PTE_RESERVED: Mask = Mask::new(7, 54);

/// Reserved for use by the Svpbmt extension, If Svpbmt is not implemented,
/// these bits remain reserved and must be zeroed by software for forward compatibility,
/// or else a page-fault exception is raised.
pub const PTE_PBMT: Mask = Mask::new(2, 61);

/// Reserved for use by the Svnapot extension, if Svnapot is not implemented,
/// this bit remain reserved must be zeroed by software for forward compatibility,
/// or else a page-fault exception is raised.
pub const PTE_N: Mask = Mask::new(1, 63);

#[derive(Clone, Copy, Debug)]
pub struct PageLevel {
    vpn: Mask,
    pte_ppn: Mask,
    pa_ppn: Mask,
    page_offset: Mask,
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PTE(usize);

impl PTE {
    /// Physical address that the PTE points to
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from(PTE_PPN.get(self.0) << PAGE_OFFSET.width())
    }

    /// The flags of a PTE
    pub fn flags(&self) -> usize {
        PTE_FLAGS.get(self.0)
    }

    /// PTE is valid
    pub fn valid(&self) -> bool {
        PTE_V.get(self.0) == 1
    }

    /// Page is readable
    pub fn readable(&self) -> bool {
        PTE_R.get(self.0) == 1
    }

    /// Page is writable
    pub fn writable(&self) -> bool {
        PTE_W.get(self.0) == 1
    }

    /// Page is executable
    pub fn executable(&self) -> bool {
        PTE_X.get(self.0) == 1
    }

    /// When RWX is 0b000, the PTE is a pointer to the next level page table;
    /// Otherwise, it is a leaf PTE.
    pub fn xwr(&self) -> usize {
        PTE_XWR.get(self.0)
    }

    /// Page is accessible to mode U.
    /// With `SUM` bit set in `sstatus`, S mode may also access pages with `U = 1`.
    /// S mode may not execute code on page with `U = 1`
    pub fn user(&self) -> bool {
        PTE_U.get(self.0) == 1
    }

    /// Page is a global mapping, which exist in all address spaces
    pub fn global(&self) -> bool {
        PTE_G.get(self.0) == 1
    }

    /// The page has been read, write, or fetched from since the last time `A` was cleared
    pub fn accessed(&self) -> bool {
        PTE_A.get(self.0) == 1
    }

    /// The page has been written since the last time `D` was cleared
    pub fn dirty(&self) -> bool {
        PTE_D.get(self.0) == 1
    }

    /// Reserved for S mode software use
    pub fn rsw(&self) -> usize {
        PTE_RSW.get(self.0)
    }
}

impl From<usize> for PTE {
    fn from(addr: usize) -> Self {
        PTE(addr)
    }
}

impl From<PTE> for usize {
    fn from(pte: PTE) -> usize {
        pte.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub const fn null() -> Self {
        PhysAddr(0)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub const fn page_offset(&self) -> usize {
        PAGE_OFFSET.get(self.0)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        PhysAddr(self.0 as usize + rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        PhysAddr(self.0.saturating_sub(rhs))
    }
}

// TODO: check address constraints
impl From<usize> for PhysAddr {
    fn from(addr: usize) -> Self {
        PhysAddr(addr)
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> usize {
        addr.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub const fn null() -> Self {
        VirtAddr(0)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub const fn page_offset(&self) -> usize {
        PAGE_OFFSET.get(self.0)
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        VirtAddr(self.0 as usize + rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        VirtAddr(self.0.saturating_sub(rhs))
    }
}

// TODO: check address constraints
impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        VirtAddr(addr)
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> usize {
        addr.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTable<T: PagingSchema> {
    table: [PTE; PAGE_SIZE / size_of::<PTE>()],
    schema: PhantomData<T>,
}

impl<T: PagingSchema + 'static> PageTable<T> {
    pub unsafe fn from_pa(pa: PhysAddr) -> &'static Self {
        &*(usize::from(pa) as *const PageTable<T>)
    }

    // TODO: return PTE instead to get flags
    pub fn walk(
        &self,
        va: VirtAddr,
        _alloc: bool, // TODO: Implement page allocation
    ) -> Result<PhysAddr, PageTableError> {
        if va >= T::max_va() {
            return Err(PageTableError::InvalidVirtualAddress);
        }

        let mut cur_pt = self;
        let mut pa = PhysAddr::null();
        let mut offset = va.page_offset();

        for level in T::page_levels().iter().rev() {
            let pte = cur_pt[level.vpn.get(va.into())];

            if !pte.valid() {
                return Err(PageTableError::InvalidPTE);
            }

            if pte.xwr() == 0b000 {
                pa = PhysAddr::from(level.pa_ppn.fill(level.pte_ppn.get(pte.into())));
                offset = level.page_offset.get(va.into());
                break;
            }

            if !pte.readable() {
                return Err(PageTableError::AccessDenied);
            }

            pa = pte.addr();

            cur_pt = unsafe { PageTable::from_pa(pa) };
        }

        assert!(pa != PhysAddr::null());
        Ok(pa + offset)
    }
}

impl<T: PagingSchema> Index<usize> for PageTable<T> {
    type Output = PTE;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PageTableError {
    InvalidVirtualAddress,
    InvalidPTE,
    AccessDenied,
}
