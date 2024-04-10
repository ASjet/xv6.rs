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
    /// Create a PTE points to `pa`
    #[inline]
    pub fn new(pa: PhysAddr, flags: usize) -> PTE {
        PTE(PTE_PPN.fill(PA_PPN.get(pa.into())) | PTE_FLAGS.fill(flags))
    }

    /// Physical address that the PTE points to
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from(PTE_PPN.get(self.0) << PAGE_OFFSET.width())
    }

    #[inline]
    pub fn set_addr(&mut self, addr: PhysAddr) {
        self.0 = PTE_PPN.set(self.0, PA_PPN.get(addr.into()));
    }

    /// The flags of a PTE
    #[inline]
    pub fn flags(&self) -> usize {
        PTE_FLAGS.get(self.0)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: usize) {
        self.0 = PTE_FLAGS.set_all(self.0) | flags;
    }

    /// PTE is valid
    #[inline]
    pub fn valid(&self) -> bool {
        PTE_V.get(self.0) == 1
    }

    /// Page is readable
    #[inline]
    pub fn readable(&self) -> bool {
        PTE_R.get(self.0) == 1
    }

    #[inline]
    pub fn set_readable(&mut self, readable: bool) {
        self.0 = PTE_R.set(self.0, readable as usize);
    }

    /// Page is writable
    #[inline]
    pub fn writable(&self) -> bool {
        PTE_W.get(self.0) == 1
    }

    #[inline]
    pub fn set_writable(&mut self, writable: bool) {
        self.0 = PTE_W.set(self.0, writable as usize);
    }

    /// Page is executable
    #[inline]
    pub fn executable(&self) -> bool {
        PTE_X.get(self.0) == 1
    }

    #[inline]
    pub fn set_executable(&mut self, executable: bool) {
        self.0 = PTE_X.set(self.0, executable as usize);
    }

    /// When RWX is 0b000, the PTE is a pointer to the next level page table;
    /// Otherwise, it is a leaf PTE.
    #[inline]
    pub fn xwr(&self) -> usize {
        PTE_XWR.get(self.0)
    }

    #[inline]
    pub fn set_xwr(&mut self, xwr: usize) {
        self.0 = PTE_XWR.set(self.0, xwr);
    }

    /// Page is accessible to mode U.
    /// With `SUM` bit set in `sstatus`, S mode may also access pages with `U = 1`.
    /// S mode may not execute code on page with `U = 1`
    #[inline]
    pub fn user(&self) -> bool {
        PTE_U.get(self.0) == 1
    }

    #[inline]
    pub fn set_user(&mut self, user: bool) {
        self.0 = PTE_U.set(self.0, user as usize);
    }

    /// Page is a global mapping, which exist in all address spaces
    #[inline]
    pub fn global(&self) -> bool {
        PTE_G.get(self.0) == 1
    }

    #[inline]
    pub fn set_global(&mut self, global: bool) {
        self.0 = PTE_G.set(self.0, global as usize);
    }

    /// The page has been read, write, or fetched from since the last time `A` was cleared
    #[inline]
    pub fn accessed(&self) -> bool {
        PTE_A.get(self.0) == 1
    }

    #[inline]
    pub fn set_accessed(&mut self, accessed: bool) {
        self.0 = PTE_A.set(self.0, accessed as usize);
    }

    /// The page has been written since the last time `D` was cleared
    #[inline]
    pub fn dirty(&self) -> bool {
        PTE_D.get(self.0) == 1
    }

    #[inline]
    pub fn set_dirty(&mut self, dirty: bool) {
        self.0 = PTE_D.set(self.0, dirty as usize);
    }

    /// Reserved for S mode software use
    #[inline]
    pub fn rsw(&self) -> usize {
        PTE_RSW.get(self.0)
    }

    #[inline]
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
