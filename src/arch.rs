#[cfg(target_arch = "riscv64")]
mod rv64;

#[cfg(target_arch = "riscv64")]
pub use rv64::*;
