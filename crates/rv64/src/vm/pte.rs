use super::{PhysAddr, PAGE_OFFSET, PA_PPN};
use crate::Mask;
use core::fmt::Debug;

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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PTE(usize);

impl PTE {
    /// Create a PTE points to `pa`, use `new` instead setters
    #[inline]
    pub fn new(pa: PhysAddr, flags: PteFlags) -> PTE {
        PTE(PTE_PPN.fill(PA_PPN.get(pa.into())) | usize::from(flags))
    }

    /// Physical address that the PTE points to
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from(PTE_PPN.get(self.0) << PAGE_OFFSET.width())
    }

    /// The flags of a PTE
    #[inline]
    pub fn flags(&self) -> PteFlags {
        PTE_FLAGS.get(self.0).into()
    }
}

impl Debug for PTE {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PTE({:?},{:?})", self.addr(), self.flags())
    }
}

impl From<usize> for PTE {
    #[inline]
    fn from(addr: usize) -> Self {
        PTE(addr)
    }
}

impl From<PTE> for usize {
    #[inline]
    fn from(pte: PTE) -> usize {
        pte.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PteFlags(usize);

impl PteFlags {
    /// Construct with `PTE_V` set
    #[inline]
    pub const fn new() -> PteFlags {
        PteFlags(1)
    }

    /// PTE is valid
    #[inline]
    pub const fn valid(&self) -> bool {
        PTE_V.get(self.0) == 1
    }

    #[inline]
    pub fn set_valid(self, valid: bool) -> Self {
        PTE_V.set(self.0, valid as usize).into()
    }

    /// Page is readable
    #[inline]
    pub const fn readable(&self) -> bool {
        PTE_R.get(self.0) == 1
    }

    #[inline]
    pub fn set_readable(self, readable: bool) -> Self {
        PTE_R.set(self.0, readable as usize).into()
    }

    /// Page is writable
    #[inline]
    pub const fn writable(&self) -> bool {
        PTE_W.get(self.0) == 1
    }

    #[inline]
    pub fn set_writable(self, writable: bool) -> Self {
        PTE_W.set(self.0, writable as usize).into()
    }

    /// Page is executable
    #[inline]
    pub const fn executable(&self) -> bool {
        PTE_X.get(self.0) == 1
    }

    #[inline]
    pub fn set_executable(self, executable: bool) -> Self {
        PTE_X.set(self.0, executable as usize).into()
    }

    /// When RWX is 0b000, the PTE is a pointer to the next level page table;
    /// Otherwise, it is a leaf PTE.
    #[inline]
    pub const fn xwr(&self) -> usize {
        PTE_XWR.get(self.0)
    }

    #[inline]
    pub fn set_xwr(self, xwr: usize) -> Self {
        PTE_XWR.set(self.0, xwr).into()
    }

    /// Page is accessible to mode U.
    /// With `SUM` bit set in `sstatus`, S mode may also access pages with `U = 1`.
    /// S mode may not execute code on page with `U = 1`
    #[inline]
    pub const fn user(&self) -> bool {
        PTE_U.get(self.0) == 1
    }

    #[inline]
    pub fn set_user(self, user: bool) -> Self {
        PTE_U.set(self.0, user as usize).into()
    }

    /// Page is a global mapping, which exist in all address spaces
    #[inline]
    pub const fn global(&self) -> bool {
        PTE_G.get(self.0) == 1
    }

    #[inline]
    pub fn set_global(self, global: bool) -> Self {
        PTE_G.set(self.0, global as usize).into()
    }

    /// The page has been read, write, or fetched from since the last time `A` was cleared
    #[inline]
    pub const fn accessed(&self) -> bool {
        PTE_A.get(self.0) == 1
    }

    #[inline]
    pub fn set_accessed(self, accessed: bool) -> Self {
        PTE_A.set(self.0, accessed as usize).into()
    }

    /// The page has been written since the last time `D` was cleared
    #[inline]
    pub const fn dirty(&self) -> bool {
        PTE_D.get(self.0) == 1
    }

    #[inline]
    pub fn set_dirty(self, dirty: bool) -> Self {
        PTE_D.set(self.0, dirty as usize).into()
    }

    /// Reserved for S mode software use
    #[inline]
    pub const fn rsw(&self) -> usize {
        PTE_RSW.get(self.0)
    }

    #[inline]
    pub fn set_rsw(self, rsw: usize) -> Self {
        PTE_RSW.set(self.0, rsw).into()
    }
}

impl Debug for PteFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02b}{}{}{}{}{}{}{}{}",
            PTE_RSW.get(self.0),
            if PTE_D.get(self.0) == 1 { "d" } else { "-" },
            if PTE_A.get(self.0) == 1 { "a" } else { "-" },
            if PTE_G.get(self.0) == 1 { "g" } else { "-" },
            if PTE_U.get(self.0) == 1 { "u" } else { "-" },
            if PTE_X.get(self.0) == 1 { "x" } else { "-" },
            if PTE_W.get(self.0) == 1 { "w" } else { "-" },
            if PTE_R.get(self.0) == 1 { "r" } else { "-" },
            if PTE_V.get(self.0) == 1 { "v" } else { "-" },
        )
    }
}

impl From<usize> for PteFlags {
    #[inline]
    fn from(flags: usize) -> Self {
        PteFlags(PTE_FLAGS.get(flags))
    }
}

impl From<PteFlags> for usize {
    #[inline]
    fn from(value: PteFlags) -> Self {
        PTE_FLAGS.get_raw(value.0)
    }
}
