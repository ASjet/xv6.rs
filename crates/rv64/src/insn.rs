use core::mem::size_of;
use int_enum::IntEnum;

/// Machine Privileged Level
pub mod m;

/// Supervisor Privileged Level
pub mod s;

/// Unprivileged Level
pub mod u;

#[derive(Debug, IntEnum)]
#[repr(u8)]
pub enum PrivilegeLevel {
    U = 0b00,
    S = 0b01,
    Reserved = 0b10,
    M = 0b11,
}

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

    /// Set the value at the mask in the target.
    #[inline]
    pub const fn set(&self, target: usize, value: usize) -> usize {
        (target & !self.mask) | (value << self.shift)
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

    /// Get the value at the mask in the target.
    #[inline]
    pub const fn get(&self, target: usize) -> usize {
        (target & self.mask) >> self.shift
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

    /// Fill the mask with the value.
    #[inline]
    pub const fn fill(&self, value: usize) -> usize {
        self.set(0, value)
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

pub trait RegisterRW<T: From<usize> + Into<usize>> {
    /// Read the value of the register.
    fn read(&self) -> T;

    /// Write the value to the register.
    unsafe fn write(&self, value: T);

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: Mask) -> usize {
        mask.get(self.read().into())
    }

    /// Write the value of the register at the mask.
    #[inline]
    unsafe fn write_mask(&self, mask: Mask, value: usize) {
        self.write(mask.set(self.read().into(), value).into())
    }

    /// Set all bits at the mask in the register.
    #[inline]
    unsafe fn set_mask(&self, mask: Mask) {
        self.write(mask.set_all(self.read().into()).into())
    }

    /// Clear all bits at the mask in the register.
    #[inline]
    unsafe fn clear_mask(&self, mask: Mask) {
        self.write(mask.clear_all(self.read().into()).into())
    }
}

pub trait RegisterRO<T: From<usize> + Into<usize>> {
    /// Read the value of the register.
    fn read(&self) -> T;

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: Mask) -> T {
        mask.get(self.read().into()).into()
    }
}

#[macro_export]
macro_rules! mv_reg_rw {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW<usize> for $reg {
            #[inline]
            fn read(&self) -> usize {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: usize) {
                unsafe {
                    core::arch::asm!(
                        concat!("mv ", stringify!($reg), ", {}"),
                        in(reg) x
                    )
                };
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $val:ident $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy)]
        pub struct $val(usize);

        impl From<$val> for usize {
            #[inline]
            fn from(v: $val) -> usize {
                v.0
            }
        }

        impl From<usize> for $val {
            #[inline]
            fn from(v: usize) -> $val {
                $val(v)
            }
        }

        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW<$val> for $reg {
            #[inline]
            fn read(&self) -> $val {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r
                        $(,options($($options),*))?
                    )
                };
                $val(r)
            }

            #[inline]
            unsafe fn write(&self, x: $val) {
                unsafe {
                    core::arch::asm!(
                        concat!("mv ", stringify!($reg), ", {}"),
                        in(reg) x.0
                        $(,options($($options),*))?
                    )
                };
            }
        }
    };
}

#[macro_export]
macro_rules! mv_reg_ro {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO<usize> for $reg {
            #[inline]
            fn read(&self) -> usize {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $val:ident $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy)]
        pub struct $val(usize);

        impl From<$val> for usize {
            #[inline]
            fn from(v: $val) -> usize {
                v.0
            }
        }

        impl From<usize> for $val {
            #[inline]
            fn from(v: usize) -> $val {
                $val(v)
            }
        }

        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO<$val> for $reg {
            #[inline]
            fn read(&self) -> $val {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r
                        $(,options($($options),*))?
                    )
                };
                $val(r)
            }
        }
    };
}

#[macro_export]
macro_rules! csr_reg_rw {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW<usize> for $reg {
            #[inline]
            fn read(&self) -> usize {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: usize) {
                unsafe {
                    core::arch::asm!(
                        concat!("csrw ", stringify!($reg), ", {}"),
                        in(reg) x
                    )
                };
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $val:ident $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy)]
        pub struct $val(usize);

        impl From<$val> for usize {
            #[inline]
            fn from(v: $val) -> usize {
                v.0
            }
        }

        impl From<usize> for $val {
            #[inline]
            fn from(v: usize) -> $val {
                $val(v)
            }
        }

        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW<$val> for $reg {
            #[inline]
            fn read(&self) -> $val {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r
                        $(,options($($options),*))?
                    )
                };
                $val(r)
            }

            #[inline]
            unsafe fn write(&self, x: $val) {
                unsafe {
                    core::arch::asm!(
                        concat!("csrw ", stringify!($reg), ", {}"),
                        in(reg) x.0
                        $(,options($($options),*))?
                    )
                };
            }
        }
    };
}

#[macro_export]
macro_rules! csr_reg_ro {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO<usize> for $reg {
            #[inline]
            fn read(&self) -> usize {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $val:ident $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy)]
        pub struct $val(usize);

        impl From<$val> for usize {
            #[inline]
            fn from(v: $val) -> usize {
                v.0
            }
        }

        impl From<usize> for $val {
            #[inline]
            fn from(v: usize) -> $val {
                $val(v)
            }
        }

        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO<$val> for $reg {
            #[inline]
            fn read(&self) -> $val {
                let r: usize;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r
                        $(,options($($options),*))?
                    )
                };
                $val(r)
            }
        }
    };
}

#[macro_export]
macro_rules! naked_insn {
    ($(#[$m:meta])* $reg:ident $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub fn $reg() {
            unsafe { core::arch::asm!(stringify!($reg) $(,options($($options),*))?) };
        }
    };
}

#[macro_export]
macro_rules! read_linker_symbol {
    ($symbol:ident) => {
        {
            let r: usize;
            core::arch::asm!(
                concat!("lui {0}, %hi(", stringify!($symbol), ")"),
                concat!("add {0}, {0}, %lo(", stringify!($symbol), ")"),
                out(reg) r
            );
            r
        }
    };
}
