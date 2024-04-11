use crate::BitFlag;

/// Machine Privileged Level
mod m;

/// Supervisor Privileged Level
mod s;

/// Unprivileged Level
mod u;

pub use m::*;
pub use s::*;
pub use u::*;

pub trait RegisterRW<T: From<usize> + Into<usize>> {
    /// Read the value of the register.
    fn read(&self) -> T;

    /// Write the value to the register.
    unsafe fn write(&self, value: T);

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: BitFlag) -> usize {
        mask.read(self.read().into())
    }

    /// Write the value of the register at the mask.
    #[inline]
    unsafe fn write_mask(&self, mask: BitFlag, value: usize) {
        self.write(mask.write(self.read().into(), value).into())
    }

    /// Set all bits at the mask in the register.
    #[inline]
    unsafe fn set_mask(&self, mask: BitFlag) {
        self.write(mask.set(self.read().into()).into())
    }

    /// Clear all bits at the mask in the register.
    #[inline]
    unsafe fn clear_mask(&self, mask: BitFlag) {
        self.write(mask.clear(self.read().into()).into())
    }
}

pub trait RegisterRO<T: From<usize> + Into<usize>> {
    /// Read the value of the register.
    fn read(&self) -> T;

    /// Read the value of the register at the mask.
    #[inline]
    fn read_mask(&self, mask: BitFlag) -> T {
        mask.read(self.read().into()).into()
    }
}

#[macro_export]
macro_rules! mv_reg_rw {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl crate::reg::RegisterRW<usize> for $reg {
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

        impl crate::reg::RegisterRW<usize> for $reg {
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

        impl crate::reg::RegisterRW<$val> for $reg {
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

        impl crate::reg::RegisterRO<usize> for $reg {
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

        impl crate::reg::RegisterRO<$val> for $reg {
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
macro_rules! csr_set_clear {
    ($reg:ident, $setter:ident, $clear:ident, $mask:expr) => {
        impl $reg {
            #[inline]
            pub unsafe fn $setter(&self) {
                unsafe {
                    self.set_mask($mask);
                }
            }

            #[inline]
            pub unsafe fn $clear(&self) {
                unsafe {
                    self.clear_mask($mask);
                }
            }
        }
    };
}
