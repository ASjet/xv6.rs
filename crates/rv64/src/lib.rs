#![no_std]
#![feature(const_ptr_as_ref)]

pub mod insn;
pub mod reg;
pub mod vm;

use core::mem::size_of;
use int_enum::IntEnum;

const BIT_INDEX: &str = "FEDCBA9876543210";

#[derive(Clone, Copy)]
pub struct Mask {
    mask: usize,
    width: u32,
    shift: u32,
}

// TODO: add unit test
impl Mask {
    #[inline]
    pub const fn new(bit_width: usize, shift: usize) -> Mask {
        assert!(bit_width + shift <= size_of::<usize>() * 8, "Mask overflow");
        Mask {
            mask: ((1 << bit_width) - 1) << shift,
            width: bit_width as u32,
            shift: shift as u32,
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
        self.shift as usize
    }

    /// Get the mask width
    #[inline]
    pub const fn width(&self) -> usize {
        self.width as usize
    }
}

impl core::ops::BitOr for Mask {
    type Output = Mask;

    fn bitor(self, rhs: Mask) -> Mask {
        let right_most = self.shift.min(rhs.shift);
        Mask {
            mask: self.mask | rhs.mask,
            width: (self.width + self.shift).max(rhs.width + rhs.shift) - right_most,
            shift: right_most,
        }
    }
}

impl core::fmt::Display for Mask {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:016X}", self.mask)
    }
}

impl core::fmt::Debug for Mask {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "\n {0} {0} {0} {0}\n", BIT_INDEX)?;
        write!(
            f,
            "{:016b} {:16b} {:16b} {:16b}\n",
            (self.mask >> 48) & 0xFFFF,
            (self.mask >> 32) & 0xFFFF,
            (self.mask >> 16) & 0xFFFF,
            (self.mask) & 0xFFFF,
        )
    }
}

impl From<Mask> for usize {
    fn from(mask: Mask) -> usize {
        mask.mask >> mask.shift
    }
}

#[derive(Debug, IntEnum, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PrivilegeLevel {
    U = 0b00,
    S = 0b01,
    Reserved = 0b10,
    M = 0b11,
}
