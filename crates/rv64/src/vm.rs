use crate::insn::Mask;
use core::{
    fmt::Debug,
    marker::PhantomData,
    mem::size_of,
    ops::{Add, Index, IndexMut, Sub},
};
use int_enum::IntEnum;

mod sv39;
mod sv48;
mod sv57;

pub use sv39::Sv39;
pub use sv48::Sv48;
pub use sv57::Sv57;

/// The offset inside a page frame
pub const PAGE_OFFSET: Mask = Mask::new(12, 0);
pub const PA_PPN: Mask = Mask::new(44, PAGE_OFFSET.width());
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

#[derive(Clone, Copy, Debug, IntEnum, PartialEq, Eq)]
#[repr(usize)]
pub enum PageWidth {
    W4K = PAGE_OFFSET.width() + VPN_WIDTH * 0,
    W2M = PAGE_OFFSET.width() + VPN_WIDTH * 1,
    W1G = PAGE_OFFSET.width() + VPN_WIDTH * 2,
    W39 = PAGE_OFFSET.width() + VPN_WIDTH * 3,
    W48 = PAGE_OFFSET.width() + VPN_WIDTH * 4,
}

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
    /// Create a PTE points to `pa`
    pub fn new(pa: PhysAddr, flags: usize) -> PTE {
        PTE(PTE_PPN.fill(PA_PPN.get(pa.into())) | PTE_FLAGS.fill(flags))
    }

    /// Physical address that the PTE points to
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from(PTE_PPN.get(self.0) << PAGE_OFFSET.width())
    }

    pub fn set_addr(&mut self, addr: PhysAddr) {
        self.0 = PTE_PPN.set(self.0, PA_PPN.get(addr.into()));
    }

    /// The flags of a PTE
    pub fn flags(&self) -> usize {
        PTE_FLAGS.get(self.0)
    }

    pub fn set_flags(&mut self, flags: usize) {
        self.0 = PTE_FLAGS.set_all(self.0) | flags;
    }

    /// PTE is valid
    pub fn valid(&self) -> bool {
        PTE_V.get(self.0) == 1
    }

    /// Page is readable
    pub fn readable(&self) -> bool {
        PTE_R.get(self.0) == 1
    }

    pub fn set_readable(&mut self, readable: bool) {
        self.0 = PTE_R.set(self.0, readable as usize);
    }

    /// Page is writable
    pub fn writable(&self) -> bool {
        PTE_W.get(self.0) == 1
    }

    pub fn set_writable(&mut self, writable: bool) {
        self.0 = PTE_W.set(self.0, writable as usize);
    }

    /// Page is executable
    pub fn executable(&self) -> bool {
        PTE_X.get(self.0) == 1
    }

    pub fn set_executable(&mut self, executable: bool) {
        self.0 = PTE_X.set(self.0, executable as usize);
    }

    /// When RWX is 0b000, the PTE is a pointer to the next level page table;
    /// Otherwise, it is a leaf PTE.
    pub fn xwr(&self) -> usize {
        PTE_XWR.get(self.0)
    }

    pub fn set_xwr(&mut self, xwr: usize) {
        self.0 = PTE_XWR.set(self.0, xwr);
    }

    /// Page is accessible to mode U.
    /// With `SUM` bit set in `sstatus`, S mode may also access pages with `U = 1`.
    /// S mode may not execute code on page with `U = 1`
    pub fn user(&self) -> bool {
        PTE_U.get(self.0) == 1
    }

    pub fn set_user(&mut self, user: bool) {
        self.0 = PTE_U.set(self.0, user as usize);
    }

    /// Page is a global mapping, which exist in all address spaces
    pub fn global(&self) -> bool {
        PTE_G.get(self.0) == 1
    }

    pub fn set_global(&mut self, global: bool) {
        self.0 = PTE_G.set(self.0, global as usize);
    }

    /// The page has been read, write, or fetched from since the last time `A` was cleared
    pub fn accessed(&self) -> bool {
        PTE_A.get(self.0) == 1
    }

    pub fn set_accessed(&mut self, accessed: bool) {
        self.0 = PTE_A.set(self.0, accessed as usize);
    }

    /// The page has been written since the last time `D` was cleared
    pub fn dirty(&self) -> bool {
        PTE_D.get(self.0) == 1
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.0 = PTE_D.set(self.0, dirty as usize);
    }

    /// Reserved for S mode software use
    pub fn rsw(&self) -> usize {
        PTE_RSW.get(self.0)
    }

    pub fn set_rsw(&mut self, rsw: usize) {
        self.0 = PTE_RSW.set(self.0, rsw);
    }
}

impl Debug for PTE {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PTE(0x{:x},{:010b})",
            PTE_PPN.get(self.0) << PAGE_OFFSET.width(),
            self.flags()
        )
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

    pub const fn page_roundup(&self) -> PhysAddr {
        PhysAddr(PA_PPN.get(self.0 + PAGE_SIZE - 1) << PA_PPN.shift())
    }

    pub const fn page_rounddown(&self) -> PhysAddr {
        PhysAddr(PA_PPN.get(self.0) << PA_PPN.shift())
    }

    pub unsafe fn memset<T: Sized + Copy>(&self, value: T, len: usize) {
        for ptr in (self.0..self.0 + len).step_by(size_of::<T>()) {
            *(ptr as *mut T) = value;
        }
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub const unsafe fn as_ref<T>(&self) -> Option<&'static T> {
        self.as_ptr::<T>().as_ref()
    }

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

    pub const fn page_roundup(&self) -> VirtAddr {
        VirtAddr(PA_PPN.get(self.0 + PAGE_SIZE - 1) << PA_PPN.shift())
    }

    pub const fn page_rounddown(&self) -> VirtAddr {
        VirtAddr(PA_PPN.get(self.0) << PA_PPN.shift())
    }

    pub const fn page_offset(&self) -> usize {
        PAGE_OFFSET.get(self.0)
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub const unsafe fn as_ref<T>(&self) -> Option<&'static T> {
        self.as_ptr::<T>().as_ref()
    }

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
    pub fn virt_to_phys(&self, va: VirtAddr) -> Result<PhysAddr, PageTableError> {
        if va >= T::max_va() {
            return Err(PageTableError::InvalidVirtualAddress);
        }

        let mut cur_pt = self;
        let mut pa = PhysAddr::null();
        let mut offset = va.page_offset();

        for (l, level) in T::page_levels().iter().enumerate().rev() {
            let pte = cur_pt[level.vpn.get(va.into())];

            if !pte.valid() {
                return Err(PageTableError::InvalidPTE(l, pte));
            }

            pa = PhysAddr::from(level.pa_ppn.fill(level.pte_ppn.get(pte.into())));

            if pte.xwr() != 0b000 {
                if !pte.readable() {
                    return Err(PageTableError::InvalidPTE(l, pte));
                }
                offset = level.page_offset.get(va.into());
                break;
            }

            cur_pt = unsafe {
                pte.addr()
                    .as_ref()
                    .ok_or(PageTableError::InvalidPTE(l, pte))?
            }
        }

        assert!(pa != PhysAddr::null());
        Ok(pa + offset)
    }

    pub unsafe fn walk(
        &mut self,
        va: VirtAddr,
        level: usize,
        alloc: Option<&(impl PageAllocator + Sync + Send)>,
    ) -> Result<(&'static PageLevel, &mut PTE), PageTableError> {
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

            if pte.valid() {
                if pte.xwr() != 0b000 {
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
                        *pte = PTE::new(page, PTE_V.get(PTE_FLAGS.mask()));
                    }
                } else {
                    return Err(PageTableError::InvalidPTE(l, pte.clone()));
                }
            }

            cur_pt = unsafe {
                pte.addr()
                    .as_mut()
                    .ok_or(PageTableError::InvalidPTE(l, *pte))?
            }
        }

        return Err(PageTableError::InvalidPageTable);
    }

    pub unsafe fn map_pages(
        &mut self,
        va: VirtAddr,
        size: usize,
        pa: PhysAddr,
        perm: usize,
        allocator: &(impl PageAllocator + Sync + Send),
    ) -> Result<(), PageTableError> {
        if size == 0 {
            return Err(PageTableError::InvalidMapSize);
        }

        let end = VirtAddr::from(size - 1).page_rounddown().into();

        for offset in (0..=end).step_by(PAGE_SIZE) {
            let (_, pte) = self.walk(va + offset, 0, Some(allocator))?;
            if pte.valid() {
                return Err(PageTableError::DuplicateMapping(0, *pte));
            }
            *pte = PTE::new(pa + offset, PTE_FLAGS.fill(PTE_V.set(perm, 1)));
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
            if pte.valid() {
                write!(f, "{} {:offset$}PTE[{}] = {:?}\n", cur_depth, "", i, pte)?;
                if pte.xwr() == 0b000 {
                    let pt = unsafe { pte.addr().as_mut::<Self>().unwrap() };
                    if cur_depth < max_depth {
                        pt.dump(f, cur_depth + 1, max_depth)?;
                    }
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
