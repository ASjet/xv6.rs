use super::def;
use crate::mem::alloc::ALLOCATOR;
use crate::println;
use core::ptr::addr_of;
use rv64::vm::{self, PageTable, PagingSchema, PhysAddr, Sv39, VirtAddr, PAGE_SIZE};

extern "C" {
    #[link_name = "_stext"]
    static _text_start: u8;
    #[link_name = "_etext"]
    static _text_end: u8;
    #[link_name = "_sheap"]
    static _heap_start: u8;
    #[link_name = "_eheap"]
    static _heap_end: u8;
    #[link_name = "trampoline"]
    static _trampoline: u8;

}

static mut KPGTBL: *mut PageTable<Sv39> = core::ptr::null_mut();

pub fn heap_range() -> (PhysAddr, PhysAddr) {
    unsafe {
        let start = &_heap_start as *const u8 as usize;
        let end = &_heap_end as *const u8 as usize;
        (PhysAddr::from(start), PhysAddr::from(end))
    }
}

pub fn init() {
    let perm_rw: usize = (vm::PTE_R | vm::PTE_W).into();
    let perm_rx: usize = (vm::PTE_R | vm::PTE_X).into();
    unsafe {
        let heap_start = &_heap_start as *const u8 as usize;
        let text_start = &_text_start as *const u8 as usize; // Should equal to def::KERNEL_BASE
        let text_end = &_text_end as *const u8 as usize;
        let trampoline = &_trampoline as *const u8 as usize;

        let page = ALLOCATOR.kalloc().expect("kalloc kpgtbl failed");
        page.memset(0x0usize, ALLOCATOR.page_size());
        let kpt = PageTable::<Sv39>::from_pa(page);

        // Erase the static attribute from ALLOCATOR
        let alloc = &*addr_of!(ALLOCATOR);

        let mut map_pages_log =
            |name: &str, va: usize, size: usize, pa: usize, perm: usize, err_msg: &str| {
                let va = VirtAddr::from(va);
                let pa = PhysAddr::from(pa);
                kpt.map_pages(va, size, pa, perm, alloc).expect(err_msg);
                println!(
                    "map 0x{:x} {:?} => {:?} as {}({:03b})",
                    size, pa, va, name, perm
                );
            };

        // UART registers
        map_pages_log(
            "UART0",
            def::UART0,
            PAGE_SIZE,
            def::UART0,
            perm_rw,
            "map UART0 failed",
        );

        // virtio mmio disk interface
        map_pages_log(
            "VIRTIO",
            def::VIRTIO0,
            PAGE_SIZE,
            def::VIRTIO0,
            perm_rw,
            "map VIRTIO0 failed",
        );

        // PLIC
        map_pages_log(
            "PLIC",
            def::PLIC,
            0x400000,
            def::PLIC,
            perm_rw,
            "map PLIC failed",
        );

        // map kernel text executable and read-only.
        map_pages_log(
            "kernel text",
            text_start,
            text_end - text_start,
            text_start,
            perm_rx,
            "map kernel text failed",
        );

        // map kernel data and the physical RAM we'll make use of.
        map_pages_log(
            "RAM",
            heap_start,
            def::PHY_STOP - heap_start,
            heap_start,
            perm_rw,
            "map physical RAM failed",
        );

        // map the trampoline for trap entry/exit to
        // the highest virtual address in the kernel.
        map_pages_log(
            "trampoline",
            usize::from(Sv39::max_va()) - PAGE_SIZE,
            PAGE_SIZE,
            trampoline,
            perm_rx,
            "map trampoline failed",
        );

        KPGTBL = kpt;

        // TODO: proc::map_stacks()
    }
}
