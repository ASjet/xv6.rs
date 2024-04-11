use super::def;
use crate::println;
use crate::{mem::alloc::ALLOCATOR, proc};
use core::ptr::addr_of;
use rv64::{
    insn, reg,
    vm::{PhysAddr, PteFlags, VirtAddr},
};

macro_rules! addr_reader {
    ($fn_name:ident, $sym_name:ident) => {
        #[inline]
        pub fn $fn_name() -> usize {
            extern "C" {
                static $sym_name: u8;
            }
            unsafe { addr_of!($sym_name) as usize }
        }
    };
}

addr_reader!(text_start, _stext);
addr_reader!(text_end, _etext);
addr_reader!(heap_start, _sheap);
addr_reader!(heap_end, _eheap);
addr_reader!(stack_start, _sstack);
addr_reader!(stack_end, _estack);
addr_reader!(trampoline, trampoline);

pub type PageTable = rv64::vm::PageTable<rv64::vm::Sv39>;

static mut KPGTBL: *mut PageTable = core::ptr::null_mut();

pub fn virt_to_phys(va: usize) -> Option<usize> {
    unsafe { (*KPGTBL).virt_to_phys(va).ok().map(usize::from) }
}

pub unsafe fn walk(va: usize, alloc: bool) {
    let alloc = if alloc {
        Some(&*addr_of!(ALLOCATOR))
    } else {
        None
    };
    unsafe { (*KPGTBL).walk(va, 0, alloc).unwrap() };
}

pub unsafe fn map_pages(va: usize, size: usize, pa: usize, perm: usize) {
    let alloc = &*addr_of!(ALLOCATOR);
    (*KPGTBL)
        .map_pages(va, size, pa, perm.into(), alloc)
        .unwrap();
}

pub fn init_mapping() {
    let perm_rw = PteFlags::new().set_readable(true).set_writable(true);
    let perm_rx = PteFlags::new().set_readable(true).set_executable(true);
    unsafe {
        let text_start = text_start(); // Should equal to def::KERNEL_BASE
        let text_end = text_end();
        let heap_start = heap_start();
        let heap_end = heap_end();
        let stack_start = stack_start();
        let stack_end = stack_end();
        let trampoline = trampoline();

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
        let kpt = page.as_mut::<PageTable>().unwrap();

        // Erase the static attribute from ALLOCATOR
        let alloc = &*addr_of!(ALLOCATOR);

        let mut map_pages_log =
            |name: &str, va: usize, size: usize, pa: usize, perm: PteFlags, err_msg: &str| {
                kpt.map_pages(va, size, pa, perm, alloc).expect(err_msg);
                println!(
                    "map 0x{:08x} 0x{:010x} => 0x{:010x} as {:12}({:?})",
                    size, pa, va, name, perm
                );
            };

        // UART registers
        map_pages_log(
            "UART0",
            def::UART0,
            def::PG_SIZE,
            def::UART0,
            perm_rw,
            "map UART0 failed",
        );

        // virtio mmio disk interface
        map_pages_log(
            "VIRTIO",
            def::VIRTIO0,
            def::PG_SIZE,
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
            usize::from(PageTable::max_va()) - def::PG_SIZE,
            def::PG_SIZE,
            trampoline,
            perm_rx,
            "map trampoline failed",
        );

        // Allocate a page for each process's kernel stack.
        // Map it high in memory, followed by an invalid
        // guard page.
        proc::kstack_addrs().into_iter().for_each(|va| {
            map_pages_log(
                "kstack",
                va,
                def::PG_SIZE,
                ALLOCATOR.kalloc().expect("kalloc stack failed").into(),
                perm_rw,
                "map stack failed",
            );
        });

        KPGTBL = kpt;
    }
}

pub unsafe fn enable_paging() {
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
    unsafe { reg::satp.set(reg::SatpMode::Sv39, 0, addr) }
    insn::sfence_vma();
}
