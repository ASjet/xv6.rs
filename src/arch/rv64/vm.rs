use rv64::vm::PhysAddr;

extern "C" {
    #[link_name = "_sheap"]
    static _heap_start: u8;
    #[link_name = "_heap_size"]
    static _heap_size: u8;
}

pub fn heap_range() -> (PhysAddr, PhysAddr) {
    unsafe {
        let start = &_heap_start as *const u8 as usize;
        let end = start + &_heap_size as *const u8 as usize;
        (PhysAddr::from(start), PhysAddr::from(end))
    }
}
