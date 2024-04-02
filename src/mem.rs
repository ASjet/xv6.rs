pub mod alloc;

pub fn init() {
    alloc::init_heap();
    crate::arch::vm::init();
}

pub fn init_hart() {
    // TODO: implement me
}
