use core::arch::asm;
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
    /*  0b10 is reserved */
    M = 0b11,
}

const BIT_INDEX: &str = "FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210";

pub struct Mask {
    mask: u64,
    width: u64,
    shift: u64,
}

impl Mask {
    #[inline]
    pub const fn new(bit_width: u64, shift: u64) -> Mask {
        Mask {
            mask: ((1 << bit_width) - 1) << shift,
            width: bit_width,
            shift,
        }
    }

    /// Set the value at the mask in the target.
    #[inline]
    pub fn set(&self, target: u64, value: u64) -> u64 {
        (target & !self.mask) | (value << self.shift)
    }

    /// Set all bits at the mask in the target.
    #[inline]
    pub fn set_all(&self, target: u64) -> u64 {
        target | self.mask
    }

    /// Clear all bits at the mask in the target.
    #[inline]
    pub fn clear_all(&self, target: u64) -> u64 {
        target & !self.mask
    }

    /// Get the value at the mask in the target.
    #[inline]
    pub fn get(&self, target: u64) -> u64 {
        (target & self.mask) >> self.shift
    }

    /// Get the mask value
    #[inline]
    pub const fn mask(&self) -> u64 {
        self.mask
    }

    /// Get the mask shift
    #[inline]
    pub const fn shift(&self) -> u64 {
        self.shift
    }

    /// Get the mask width
    #[inline]
    pub const fn width(&self) -> u64 {
        self.width
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

pub trait RegisterRW {
    /// Read the value of the register.
    fn read(&self) -> u64;

    /// Write the value to the register.
    unsafe fn write(&self, value: u64);

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: Mask) -> u64 {
        mask.get(self.read())
    }

    /// Write the value of the register at the mask.
    #[inline]
    unsafe fn write_mask(&self, mask: Mask, value: u64) {
        self.write(mask.set(self.read(), value))
    }

    /// Set all bits at the mask in the register.
    #[inline]
    unsafe fn set_mask(&self, mask: Mask) {
        self.write(mask.set_all(self.read()))
    }

    /// Clear all bits at the mask in the register.
    #[inline]
    unsafe fn clear_mask(&self, mask: Mask) {
        self.write(mask.clear_all(self.read()))
    }
}

pub trait RegisterRO {
    /// Read the value of the register.
    fn read(&self) -> u64;

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: Mask) -> u64 {
        mask.get(self.read())
    }
}

#[macro_export]
macro_rules! mv_reg_rw {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe {
                    core::arch::asm!(
                        concat!("mv ", stringify!($reg), ", {}"),
                        in(reg) x
                    )
                };
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $($options:ident),*) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r,
                        options($($options),*)
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe {
                    core::arch::asm!(
                        concat!("mv ", stringify!($reg), ", {}"),
                        in(reg) x,
                        options($($options),*)
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

        impl crate::insn::RegisterRO for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
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
    ($(#[$m:meta])* $reg:ident, $($options:ident),*) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("mv {}, ", stringify!($reg)),
                        out(reg) r,
                        options($($options),*)
                    )
                };
                r
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

        impl crate::insn::RegisterRW for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe {
                    core::arch::asm!(
                        concat!("csrw ", stringify!($reg), ", {}"),
                        in(reg) x
                    )
                };
            }
        }
    };
    ($(#[$m:meta])* $reg:ident, $($options:ident),*) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRW for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r,
                        options($($options),*)
                    )
                };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe {
                    core::arch::asm!(
                        concat!("csrw ", stringify!($reg), ", {}"),
                        in(reg) x,
                        options($($options),*)
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

        impl crate::insn::RegisterRO for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
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
    ($(#[$m:meta])* $reg:ident, $($options:ident),*) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::insn::RegisterRO for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe {
                    core::arch::asm!(
                        concat!("csrr {}, ", stringify!($reg)),
                        out(reg) r,
                        options($($options),*)
                    )
                };
                r
            }
        }
    };
}

#[macro_export]
macro_rules! naked_insn {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub fn $reg() {
            unsafe { core::arch::asm!(stringify!($reg)) };
        }
    };
    ($(#[$m:meta])* $reg:ident, $($options:ident),*) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub fn $reg() {
            unsafe { core::arch::asm!(stringify!($reg), options($($options),*)) };
        }
    };
}
