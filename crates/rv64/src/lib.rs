#![no_std]
#![feature(const_ptr_as_ref)]

pub mod insn;
pub mod reg;
pub mod vm;

use core::mem::size_of;
use int_enum::IntEnum;

const BIT_INDEX: &str = "FEDCBA9876543210";

#[derive(Clone, Copy)]
pub struct BitFlag {
    bits: usize,
    width: u32,
    shift: u32,
}

// TODO: add unit test
impl BitFlag {
    #[inline]
    pub const fn new(bit_width: usize, shift: usize) -> BitFlag {
        assert!(bit_width + shift <= size_of::<usize>() * 8, "Mask overflow");
        BitFlag {
            bits: ((1 << bit_width) - 1) << shift,
            width: bit_width as u32,
            shift: shift as u32,
        }
    }

    /// Read the value at the mask in the target.
    #[inline]
    pub const fn read(&self, target: usize) -> usize {
        (target & self.bits) >> self.shift
    }

    /// Equal to `target & mask`
    #[inline]
    pub const fn mask(&self, target: usize) -> usize {
        target & self.bits
    }

    /// Write value at the mask in the target.
    #[inline]
    pub const fn write(&self, target: usize, value: usize) -> usize {
        (target & !self.bits) | ((value << self.shift) & self.bits)
    }

    /// Mask the value with the shift of the bitflag.
    #[inline]
    pub const fn make(&self, value: usize) -> usize {
        self.mask(value << self.shift)
    }

    /// Set all bits at the mask in the target.
    #[inline]
    pub const fn set(&self, target: usize) -> usize {
        target | self.bits
    }

    /// Clear all bits at the mask in the target.
    #[inline]
    pub const fn clear(&self, target: usize) -> usize {
        target & !self.bits
    }

    /// Get the bits of mask
    #[inline]
    pub const fn bits(&self) -> usize {
        self.bits
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

impl core::ops::BitOr for BitFlag {
    type Output = BitFlag;

    fn bitor(self, rhs: BitFlag) -> BitFlag {
        let right_most = self.shift.min(rhs.shift);
        BitFlag {
            bits: self.bits | rhs.bits,
            width: (self.width + self.shift).max(rhs.width + rhs.shift) - right_most,
            shift: right_most,
        }
    }
}

impl core::fmt::Display for BitFlag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:016X}", self.bits)
    }
}

impl core::fmt::Debug for BitFlag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "\n {0} {0} {0} {0}\n", BIT_INDEX)?;
        write!(
            f,
            "{:016b} {:16b} {:16b} {:16b}\n",
            (self.bits >> 48) & 0xFFFF,
            (self.bits >> 32) & 0xFFFF,
            (self.bits >> 16) & 0xFFFF,
            (self.bits) & 0xFFFF,
        )
    }
}

impl From<BitFlag> for usize {
    fn from(mask: BitFlag) -> usize {
        mask.bits >> mask.shift
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

pub trait BitFlagOps: Into<usize> {
    #[inline]
    fn read_mask(self, mask: &BitFlag) -> usize {
        mask.read(self.into())
    }

    #[inline]
    fn write_mask(self, mask: &BitFlag, value: usize) -> usize {
        mask.write(self.into(), value)
    }

    #[inline]
    fn set_mask(self, mask: &BitFlag) -> usize {
        mask.set(self.into())
    }

    #[inline]
    fn clear_mask(self, mask: &BitFlag) -> usize {
        mask.clear(self.into())
    }
}

impl BitFlagOps for usize {}
