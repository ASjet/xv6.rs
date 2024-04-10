use crate::Mask;
use core::{
    fmt::Debug,
    marker::PhantomData,
    mem::size_of,
    ops::{Add, Index, IndexMut, Sub},
};
use int_enum::IntEnum;

mod pte;
mod schema;

pub use pte::*;
pub use schema::*;

/// The offset inside a page frame
pub const PAGE_OFFSET: Mask = Mask::new(12, 0);
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET.width();
pub const PA_PPN: Mask = Mask::new(44, PAGE_OFFSET.width());
pub const VPN_WIDTH: usize = PAGE_OFFSET.width() - 3; // sizeof::<usize>() == 2^3

#[derive(Clone, Copy, Debug, IntEnum, PartialEq, Eq)]
#[repr(usize)]
pub enum PageWidth {
    W4K = PAGE_OFFSET.width() + VPN_WIDTH * 0,
    W2M = PAGE_OFFSET.width() + VPN_WIDTH * 1,
    W1G = PAGE_OFFSET.width() + VPN_WIDTH * 2,
    W39 = PAGE_OFFSET.width() + VPN_WIDTH * 3,
    W48 = PAGE_OFFSET.width() + VPN_WIDTH * 4,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(usize);

impl PhysAddr {
    #[inline]
    pub const fn null() -> Self {
        PhysAddr(0)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn page_offset(&self) -> usize {
        PAGE_OFFSET.get(self.0)
    }

    #[inline]
    pub const fn page_roundup(&self) -> PhysAddr {
        PhysAddr(PA_PPN.get(self.0 + PAGE_SIZE - 1) << PA_PPN.shift())
    }

    #[inline]
    pub const fn page_rounddown(&self) -> PhysAddr {
        PhysAddr(PA_PPN.get(self.0) << PA_PPN.shift())
    }

    pub unsafe fn memset<T: Sized + Copy>(&self, value: T, len: usize) {
        for ptr in (self.0..self.0 + len).step_by(size_of::<T>()) {
            *(ptr as *mut T) = value;
        }
    }

    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub const unsafe fn as_ref<T>(&self) -> Option<&'static T> {
        self.as_ptr::<T>().as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<T>(&self) -> Option<&'static mut T> {
        self.as_mut_ptr::<T>().as_mut()
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysAddr(0x{:x})", self.0)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        PhysAddr(self.0 as usize + rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: usize) -> Self {
        PhysAddr(self.0.saturating_sub(rhs))
    }
}

// TODO: check address constraints
impl From<usize> for PhysAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        PhysAddr(addr)
    }
}

impl<T> From<*const T> for PhysAddr {
    #[inline]
    fn from(addr: *const T) -> Self {
        PhysAddr(addr as usize)
    }
}

impl<T> From<*mut T> for PhysAddr {
    #[inline]
    fn from(addr: *mut T) -> Self {
        PhysAddr(addr as usize)
    }
}

impl From<PhysAddr> for usize {
    #[inline]
    fn from(addr: PhysAddr) -> usize {
        addr.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(usize);

impl VirtAddr {
    #[inline]
    pub const fn null() -> Self {
        VirtAddr(0)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn page_roundup(&self) -> VirtAddr {
        VirtAddr(PA_PPN.get(self.0 + PAGE_SIZE - 1) << PA_PPN.shift())
    }

    #[inline]
    pub const fn page_rounddown(&self) -> VirtAddr {
        VirtAddr(PA_PPN.get(self.0) << PA_PPN.shift())
    }

    #[inline]
    pub const fn page_offset(&self) -> usize {
        PAGE_OFFSET.get(self.0)
    }

    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub const unsafe fn as_ref<T>(&self) -> Option<&'static T> {
        self.as_ptr::<T>().as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<T>(&self) -> Option<&'static mut T> {
        self.as_mut_ptr::<T>().as_mut()
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr(0x{:x})", self.0)
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        VirtAddr(self.0 as usize + rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: usize) -> Self {
        VirtAddr(self.0.saturating_sub(rhs))
    }
}

// TODO: check address constraints
impl From<usize> for VirtAddr {
    #[inline]
    fn from(addr: usize) -> Self {
        VirtAddr(addr)
    }
}

impl From<VirtAddr> for usize {
    #[inline]
    fn from(addr: VirtAddr) -> usize {
        addr.0
    }
}

/// The PageAllocator need interior mutable it's states
pub trait PageAllocator {
    unsafe fn palloc(&self, page_width: PageWidth) -> Option<PhysAddr>;
    unsafe fn pfree(&self, page: PhysAddr);
}

#[derive(Clone)]
#[repr(transparent)]
pub struct PageTable<T: PagingSchema> {
    table: [PTE; PAGE_SIZE / size_of::<PTE>()],
    schema: PhantomData<T>,
}

impl<T: PagingSchema + 'static> PageTable<T> {
    #[inline]
    pub fn max_va() -> VirtAddr {
        T::max_va()
    }

    pub fn virt_to_phys(&self, va: impl Into<VirtAddr>) -> Result<PhysAddr, PageTableError> {
        let va = va.into();
        if va >= T::max_va() {
            return Err(PageTableError::InvalidVirtualAddress);
        }

        let mut cur_pt = self;
        let mut pa = PhysAddr::null();
        let mut offset = va.page_offset();

        for (l, level) in T::page_levels().iter().enumerate().rev() {
            let pte = cur_pt[level.vpn.get(va.into())];

            let flags = pte.flags();
            if !flags.valid() {
                return Err(PageTableError::InvalidPTE(l, pte));
            }

            pa = PhysAddr::from(level.pa_ppn.fill(level.pte_ppn.get(pte.into())));

            if flags.xwr() != 0b000 {
                if !flags.readable() {
                    return Err(PageTableError::InvalidPTE(l, pte));
                }
                offset = level.page_offset.get(va.into());
                break;
            }

            cur_pt = unsafe { pte.addr().as_ref() }.ok_or(PageTableError::InvalidPTE(l, pte))?
        }

        assert!(pa != PhysAddr::null());
        Ok(pa + offset)
    }

    pub unsafe fn walk(
        &mut self,
        va: impl Into<VirtAddr>,
        level: usize,
        alloc: Option<&(impl PageAllocator + Sync + Send)>,
    ) -> Result<(&'static PageLevel, &mut PTE), PageTableError> {
        let va = va.into();
        if va >= T::max_va() {
            return Err(PageTableError::InvalidVirtualAddress);
        }

        if level >= T::page_levels().len() {
            return Err(PageTableError::InvalidPageLevel);
        }

        let mut cur_pt = self;

        for (l, pl) in T::page_levels().iter().enumerate().rev() {
            let pte = &mut cur_pt[pl.vpn.get(va.into())];

            if l == level {
                return Ok((pl, pte));
            }

            let flags = pte.flags();

            if flags.valid() {
                if flags.xwr() != 0b000 {
                    return Ok((pl, pte));
                }
            } else {
                if let Some(allocator) = alloc {
                    let page_width = if l == level {
                        PageWidth::try_from(pl.page_offset.width()).expect("invalid page width")
                    } else {
                        PageWidth::W4K
                    };
                    unsafe {
                        let page = allocator
                            .palloc(page_width)
                            .ok_or(PageTableError::AllocFailed)?;
                        page.memset(0usize, 1 << usize::from(page_width));
                        *pte = PTE::new(page, PteFlags::new());
                    }
                } else {
                    return Err(PageTableError::InvalidPTE(l, pte.clone()));
                }
            }

            cur_pt = unsafe { pte.addr().as_mut() }.ok_or(PageTableError::InvalidPTE(l, *pte))?
        }

        return Err(PageTableError::InvalidPageTable);
    }

    pub unsafe fn map_pages(
        &mut self,
        va: impl Into<VirtAddr>,
        size: usize,
        pa: impl Into<PhysAddr>,
        perm: PteFlags,
        allocator: &(impl PageAllocator + Sync + Send),
    ) -> Result<(), PageTableError> {
        let va = va.into();
        let pa = pa.into();
        if size == 0 {
            return Err(PageTableError::InvalidMapSize);
        }

        let end = VirtAddr::from(size - 1).page_rounddown().into();
        let perm = perm.set_valid(true);

        for offset in (0..=end).step_by(PAGE_SIZE) {
            let (_, pte) = self.walk(va + offset, 0, Some(allocator))?;
            if pte.flags().valid() {
                return Err(PageTableError::DuplicateMapping(0, *pte));
            }
            *pte = PTE::new(pa + offset, perm);
        }

        Ok(())
    }

    pub fn dump(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        cur_depth: usize,
        max_depth: usize,
    ) -> core::fmt::Result {
        let offset = cur_depth * 4;
        for (i, pte) in self.table.iter().enumerate() {
            let flags = pte.flags();
            if flags.valid() {
                write!(f, "{} {:offset$}PTE[{}] = {:?}\n", cur_depth, "", i, pte)?;
                if flags.xwr() == 0b000 && cur_depth < max_depth {
                    unsafe { pte.addr().as_mut::<Self>() }.unwrap().dump(
                        f,
                        cur_depth + 1,
                        max_depth,
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl<T: PagingSchema> Index<usize> for PageTable<T> {
    type Output = PTE;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl<T: PagingSchema> IndexMut<usize> for PageTable<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

impl<T: PagingSchema + 'static> Debug for PageTable<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\n")?;
        self.dump(f, 0, 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTableError {
    InvalidPageTable,
    InvalidPTE(usize, PTE),
    InvalidVirtualAddress,
    InvalidPageLevel,
    AllocFailed,
    InvalidMapSize,
    DuplicateMapping(usize, PTE),
}
