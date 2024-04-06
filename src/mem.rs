pub mod alloc;

pub fn init() {
    alloc::init_heap();
    crate::arch::vm::init_mapping();
}

pub fn init_hart() {
    crate::arch::vm::enable_paging();
}