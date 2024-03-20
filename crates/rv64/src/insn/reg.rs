use super::Register;
use core::arch::asm;

macro_rules! mv_reg {
    ($(#[$m:meta])* $reg:ident) => {
        $(#[$m])*
        #[allow(non_camel_case_types)]
        pub struct $reg;

        impl Register for $reg {
            #[inline]
            fn read(&self) -> u64 {
                let r: u64;
                unsafe { asm!(concat!("mv {}, ", stringify!($reg)), out(reg) r) };
                r
            }

            #[inline]
            unsafe fn write(&self, x: u64) {
                unsafe { asm!(concat!("mv ", stringify!($reg), ", {}"), in(reg) x) };
            }
        }
    };
}

mv_reg!(
    /// Stack pointer
    sp
);

mv_reg!(
    /// Thread pointer, which should holds this core's hartid
    tp
);

mv_reg!(
    /// Return address
    ra
);
