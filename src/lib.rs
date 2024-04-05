#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]
#![allow(dead_code)]

pub mod arch;
pub mod io;
pub mod mem;
pub mod print;
pub mod proc;
pub mod spinlock;

/// Should be equal to _max_hart_id
pub const NCPU: usize = 8;

/// Maximum supported number of processes
pub const NPROC: usize = NCPU * 2; // TODO: increase latter

// TODO: detect and set `NCPU` and `NPROC`
