#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

pub mod arch;
pub mod io;
pub mod print;
pub mod proc;
pub mod spinlock;

pub const NCPU: usize = 8;
