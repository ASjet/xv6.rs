use crate::arch;

pub fn kstack_addrs() -> [usize; crate::NPROC] {
    core::array::from_fn(arch::def::kstack)
}
