#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[inline]
pub fn exit_qemu(exit_code: QemuExitCode) {
    PortIndex::ISADebugExit.write(exit_code as u32);
}
