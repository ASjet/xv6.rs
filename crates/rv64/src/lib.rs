#![no_std]
#![feature(const_ptr_as_ref)]

pub mod insn;
pub mod vm;

use core::mem::size_of;

const BIT_INDEX: &str = "FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210";

#[derive(Clone, Copy)]
pub struct Mask {
    mask: usize,
    width: usize,
    shift: usize,
}

// TODO: add unit test
impl Mask {
    #[inline]
    pub const fn new(bit_width: usize, shift: usize) -> Mask {
        Mask {
            mask: ((1 << bit_width) - 1) << shift,
            width: bit_width,
            shift: shift,
        }
    }

    /// Get the value at the mask in the target.
    #[inline]
    pub const fn get(&self, target: usize) -> usize {
        (target & self.mask) >> self.shift
    }

    /// Equal to `target & mask`
    #[inline]
    pub const fn get_raw(&self, target: usize) -> usize {
        target & self.mask
    }

    /// Set the value at the mask in the target.
    #[inline]
    pub const fn set(&self, target: usize, value: usize) -> usize {
        (target & !self.mask) | ((value << self.shift) & self.mask)
    }

    /// Set all bits at the mask in the target.
    #[inline]
    pub const fn set_all(&self, target: usize) -> usize {
        target | self.mask
    }

    /// Clear all bits at the mask in the target.
    #[inline]
    pub const fn clear_all(&self, target: usize) -> usize {
        target & !self.mask
    }

    /// Fill the mask with the value.
    #[inline]
    pub const fn fill(&self, value: usize) -> usize {
        self.set(0, value)
    }

    /// Get the mask value
    #[inline]
    pub const fn mask(&self) -> usize {
        self.mask
    }

    /// Get the mask shift
    #[inline]
    pub const fn shift(&self) -> usize {
        self.shift
    }

    /// Get the mask width
    #[inline]
    pub const fn width(&self) -> usize {
        self.width
    }
}

impl core::ops::BitOr for Mask {
    type Output = Mask;

    fn bitor(self, rhs: Mask) -> Mask {
        Mask {
            mask: self.mask | rhs.mask,
            width: (size_of::<usize>() * 8).max(self.width + rhs.width),
            shift: self.shift.min(rhs.shift),
        }
    }
}

impl core::fmt::Display for Mask {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:016X}", self.mask)
    }
}

impl core::fmt::Debug for Mask {
    // TODO: compress sparse bits.
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "\n{BIT_INDEX}\n{:064b}\n", self.mask)
    }
}

impl From<Mask> for usize {
    fn from(mask: Mask) -> usize {
        mask.mask >> mask.shift
    }
}
