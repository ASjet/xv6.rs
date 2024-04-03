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
        // FIXME: define trampoline in link script
        // let trampoline = &_trampoline as *const u8 as usize;

        let page = ALLOCATOR.kalloc().expect("kalloc kpgtbl failed");
        page.memset(0x0usize, ALLOCATOR.page_size());
        let kpt = PageTable::<Sv39>::from_pa(page);

        // Erase the static attribute from ALLOCATOR
        let alloc = &*addr_of!(ALLOCATOR);

        // UART registers
        kpt.map_pages(
            VirtAddr::from(def::UART0),
            PAGE_SIZE,
            PhysAddr::from(def::UART0),
            perm_rw,
            alloc,
        )
        .expect("map UART0 failed");

        // virtio mmio disk interface
        kpt.map_pages(
            VirtAddr::from(def::VIRTIO0),
            PAGE_SIZE,
            PhysAddr::from(def::VIRTIO0),
            perm_rw,
            alloc,
        )
        .expect("map VIRTIO0 failed");

        // PLIC
        kpt.map_pages(
            VirtAddr::from(def::PLIC),
            0x400000,
            PhysAddr::from(def::PLIC),
            perm_rw,
            alloc,
        )
        .expect("map PLIC failed");

        // map kernel text executable and read-only.
        kpt.map_pages(
            VirtAddr::from(text_start),
            text_end - text_start,
            PhysAddr::from(text_start),
            perm_rx,
            alloc,
        )
        .expect("map kernel text failed");

        // map kernel data and the physical RAM we'll make use of.
        kpt.map_pages(
            VirtAddr::from(heap_start),
            def::PHY_STOP - heap_start,
            PhysAddr::from(heap_start),
            perm_rw,
            alloc,
        )
        .expect("map physical RAM failed");

        // map the trampoline for trap entry/exit to
        // the highest virtual address in the kernel.
        // kpt.map_pages(
        //     Sv39::max_va() - PAGE_SIZE,
        //     PAGE_SIZE,
        //     PhysAddr::from(trampoline),
        //     perm_rx,
        //     alloc,
        // )
        // .expect("map trampoline failed");

        KPGTBL = kpt;

        // TODO: proc::map_stacks()
    }
}
