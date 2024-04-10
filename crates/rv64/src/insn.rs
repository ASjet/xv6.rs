/// Machine Privileged Level
mod m;

/// Supervisor Privileged Level
mod s;

/// Unprivileged Level
mod u;

pub use m::*;
pub use s::*;
pub use u::*;

#[macro_export]
macro_rules! instruction {
    ($(#[$m:meta])* unsafe $reg_fn:ident, $asm:expr $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[inline]
        pub unsafe fn $reg_fn() {
            unsafe { core::arch::asm!($asm $(,options($($options),*))?) };
        }
    };
    ($(#[$m:meta])* $reg_fn:ident, $asm:expr $(,$($options:ident),*)?) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        #[inline]
        pub fn $reg_fn() {
            unsafe { core::arch::asm!($asm $(,options($($options),*))?) };
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
