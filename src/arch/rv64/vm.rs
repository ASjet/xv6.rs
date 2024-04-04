use super::def;
use crate::println;
use crate::{mem::alloc::ALLOCATOR, proc};
use core::ptr::addr_of;
use rv64::{
    insn::s,
    vm::{self, PageTable, PagingSchema, PhysAddr, Sv39, VirtAddr, PAGE_SIZE, PA_PPN},
};

extern "C" {
    #[link_name = "_stext"]
    static _text_start: u8;
    #[link_name = "_etext"]
    static _text_end: u8;
    #[link_name = "_sheap"]
    static _heap_start: u8;
    #[link_name = "_eheap"]
    static _heap_end: u8;
    #[link_name = "_sstack"]
    static _stack_start: u8;
    #[link_name = "_estack"]
    static _stack_end: u8;
    #[link_name = "trampoline"]
    static _trampoline: u8;
}

static mut KPGTBL: *mut PageTable<Sv39> = core::ptr::null_mut();

pub fn heap_range() -> (PhysAddr, PhysAddr) {
    unsafe {
        let start = addr_of!(_heap_start) as usize;
        let end = addr_of!(_heap_end) as usize;
        (
            PhysAddr::from(start),
            PhysAddr::from(end).page_rounddown() - 1,
        )
    }
}

pub fn init_mapping() {
    let perm_rw: usize = (vm::PTE_R | vm::PTE_W).mask();
    let perm_rx: usize = (vm::PTE_R | vm::PTE_X).mask();
    unsafe {
        let text_start = addr_of!(_text_start) as usize; // Should equal to def::KERNEL_BASE
        let text_end = addr_of!(_text_end) as usize;
        let heap_start = addr_of!(_heap_start) as usize;
        let heap_end = addr_of!(_heap_end) as usize;
        let stack_start = addr_of!(_stack_start) as usize;
        let stack_end = addr_of!(_stack_end) as usize;
        let trampoline = addr_of!(_trampoline) as usize;

        println!(
            "heap(0x{:x}): {:?} => {:?}\nstack(0x{:x}): {:?} => {:?}",
            heap_end - heap_start,
            PhysAddr::from(heap_start),
            PhysAddr::from(heap_end),
            stack_start - stack_end,
            PhysAddr::from(stack_end),
            PhysAddr::from(stack_start),
        );

        let page = ALLOCATOR.kalloc().expect("kalloc kpgtbl failed");
        page.memset(0x0usize, ALLOCATOR.page_size());
        let kpt = page.as_mut::<PageTable<Sv39>>().unwrap();

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
            text_end,
            stack_start - text_end,
            text_end,
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

        // TODO: proc::map_stacks()

        KPGTBL = kpt;
    }
}

pub fn enable_paging() {
    let addr = unsafe { KPGTBL as usize };
    // make sure for RAM mapping is working
    assert_eq!(
        usize::from(addr),
        usize::from(unsafe {
            (*KPGTBL)
                .virt_to_phys(VirtAddr::from(usize::from(addr)))
                .expect("virt_to_phys failed")
        })
    );
    println!(
        "enable kernel page table at 0x{:x} on hart {}",
        addr,
        super::cpuid()
    );
    unsafe { s::satp.set(s::SatpMode::Sv39, 0, PA_PPN.get(addr)) }
    s::sfence_vma();
}
