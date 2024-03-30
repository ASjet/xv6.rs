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
pub struct PPN {
    pte: Mask,
    pa: Mask,
    page_offset: Mask,
}

impl PPN {
    pub const fn new(pte: Mask, pa: Mask) -> Self {
        PPN {
            pte,
            pa,
            page_offset: Mask::new(pa.shift(), 0),
        }
    }
}

pub trait PagingSchema {
    fn max_va() -> VirtAddr;

    fn page_addr() -> &'static Mask;

    fn pte_addr() -> &'static Mask;

    fn ppn() -> &'static [PPN];

    fn va_vpn() -> &'static [Mask];
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PTE<T: PagingSchema> {
    entry: usize,
    schema: PhantomData<T>,
}

impl<T: PagingSchema> PTE<T> {
    /// Physical address that the PTE points to
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from(T::page_addr().fill(T::pte_addr().get(self.entry)))
    }

    /// The flags of a PTE
    pub fn flags(&self) -> usize {
        PTE_FLAGS.get(self.entry)
    }

    /// PTE is valid
    pub fn valid(&self) -> bool {
        PTE_V.get(self.entry) == 1
    }

    /// Page is readable
    pub fn readable(&self) -> bool {
        PTE_R.get(self.entry) == 1
    }

    /// Page is writable
    pub fn writable(&self) -> bool {
        PTE_W.get(self.entry) == 1
    }

    /// Page is executable
    pub fn executable(&self) -> bool {
        PTE_X.get(self.entry) == 1
    }

    /// When RWX is 0b000, the PTE is a pointer to the next level page table;
    /// Otherwise, it is a leaf PTE.
    pub fn xwr(&self) -> usize {
        PTE_XWR.get(self.entry)
    }

    /// Page is accessible to mode U.
    /// With `SUM` bit set in `sstatus`, S mode may also access pages with `U = 1`.
    /// S mode may not execute code on page with `U = 1`
    pub fn user(&self) -> bool {
        PTE_U.get(self.entry) == 1
    }

    /// Page is a global mapping, which exist in all address spaces
    pub fn global(&self) -> bool {
        PTE_G.get(self.entry) == 1
    }

    /// The page has been read, write, or fetched from since the last time `A` was cleared
    pub fn accessed(&self) -> bool {
        PTE_A.get(self.entry) == 1
    }

    /// The page has been written since the last time `D` was cleared
    pub fn dirty(&self) -> bool {
        PTE_D.get(self.entry) == 1
    }

    /// Reserved for S mode software use
    pub fn rsw(&self) -> usize {
        PTE_RSW.get(self.entry)
    }

    pub fn as_usize(&self) -> usize {
        self.entry
    }
}

impl<T: PagingSchema> From<usize> for PTE<T> {
    fn from(addr: usize) -> Self {
        PTE {
            entry: addr,
            schema: PhantomData::<T>,
        }
    }
}

/*
An Sv39 address is
partitioned as shown in Figure 58. Instruction fetch addresses and load and store effective addresses,
which are 64 bits, must have bits 63–39 all equal to bit 38, or else a page-fault exception will occur.
The 27-bit VPN is translated into a 44-bit PPN via a three-level page table, while the 12-bit page offset
is untranslated.
*/

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

    pub const fn as_usize(&self) -> usize {
        self.0
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

impl From<usize> for PhysAddr {
    fn from(addr: usize) -> Self {
        PhysAddr(addr)
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

    pub const fn as_usize(&self) -> usize {
        self.0
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

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        VirtAddr(addr)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTable<T: PagingSchema> {
    table: [PTE<T>; PAGE_SIZE / size_of::<usize>()],
    schema: PhantomData<T>,
}

impl<T: PagingSchema + 'static> PageTable<T> {
    pub unsafe fn from_pa(pa: PhysAddr) -> &'static Self {
        &*(pa.as_usize() as *const PageTable<T>)
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

        for (level, vpn) in T::va_vpn().iter().enumerate().rev() {
            let pte = &cur_pt[vpn.get(va.as_usize())];

            if !pte.valid() {
                return Err(PageTableError::InvalidPTE);
            }

            let PPN {
                pte: pte_ppn,
                pa: pa_ppn,
                page_offset,
            } = T::ppn()[level];

            if pte.xwr() == 0b000 {
                pa = PhysAddr::from(pa_ppn.fill(pte_ppn.get(pte.as_usize())));
                offset = page_offset.get(va.as_usize());
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
    type Output = PTE<T>;

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
