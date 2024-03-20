pub mod csr;

const BIT_INDEX: &str = "FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210";

pub struct Mask {
    mask: u64,
    shift: u64,
}

impl Mask {
    #[inline]
    pub const fn new(bit_width: u64, shift: u64) -> Mask {
        Mask {
            mask: ((1 << bit_width) - 1) << shift,
            shift,
        }
    }

    #[inline]
    pub fn set(&self, target: u64, value: u64) -> u64 {
        (target & !self.mask) | (value << self.shift)
    }

    #[inline]
    pub fn get(&self, target: u64) -> u64 {
        (target & self.mask) >> self.shift
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
