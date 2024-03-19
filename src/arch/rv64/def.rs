#![allow(dead_code)]

pub const PGSIZE: u64 = 4096; // bytes per page
pub const PGSHIFT: u64 = 12; // bits of offset within a page

#[inline(always)]
pub const fn pgroundup(sz: u64) -> u64 {
    (sz + PGSIZE - 1) & !(PGSIZE - 1)
}

#[inline(always)]
pub const fn pgrounddown(a: u64) -> u64 {
    (a) & !(PGSIZE - 1)
}

pub const PTE_V: u64 = 1 << 0; // valid
pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;
pub const PTE_U: u64 = 1 << 4; // 1 -> user can access

#[inline(always)]
pub const fn pa2pte(pa: u64) -> u64 {
    ((pa) >> 12) << 10
}

#[inline(always)]
pub const fn pte2pa(pte: u64) -> u64 {
    ((pte) >> 10) << 12
}

#[inline(always)]
pub const fn pte_flags(pte: u64) -> u64 {
    (pte) & 0x3FF
}

pub const PXMASK: u64 = 0x1FF; // 9 bits

#[inline(always)]
pub const fn pxshift(level: u64) -> u64 {
    PGSHIFT + (9 * (level))
}

#[inline(always)]
pub const fn px(level: u64, va: u64) -> u64 {
    ((va) >> pxshift(level)) & PXMASK
}

pub const MAXVA: u64 = 1 << (9 + 9 + 9 + 12 - 1);

/// Physical memory layout

/// qemu -machine virt is set up like this,
/// based on qemu's hw/riscv/virt.c:
///
/// 00001000 -- boot ROM, provided by qemu
/// 02000000 -- CLINT
/// 0C000000 -- PLIC
/// 10000000 -- uart0
/// 10001000 -- virtio disk
/// 80000000 -- boot ROM jumps here in machine mode
///             -kernel loads the kernel here
/// unused RAM after 80000000.

/// the kernel uses physical memory thus:
/// 80000000 -- entry.S, then kernel text and data
/// end -- start of kernel page allocation area
/// PHYSTOP -- end RAM used by the kernel
pub const KERNEL_BASE: u64 = 0x80000000;
pub const PHYSTOP: u64 = KERNEL_BASE + 128 * 1024 * 1024;

/// qemu puts UART registers here in physical memory.
pub const UART0: u64 = 0x10000000;
pub const UART0_IRQ: u64 = 10;

/// virtio mmio interface
pub const VIRTIO0: u64 = 0x10001000;
pub const VIRTIO0_IRQ: u64 = 1;

/// core local interruptor (CLINT), which contains the timer.
pub const CLINT: u64 = 0x2000000;
#[inline(always)]
pub const fn clint_mtimecmp(hartid: u64) -> u64 {
    CLINT + 0x4000 + 8 * hartid
}
pub const CLINT_MTIME: u64 = CLINT + 0xBFF8; // cycles since boot.

/// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: u64 = 0x0c000000;
pub const PLIC_PRIORITY: u64 = PLIC + 0x0;
pub const PLIC_PENDING: u64 = PLIC + 0x1000;

#[inline(always)]
pub const fn plic_menable(hart: u64) -> u64 {
    PLIC + 0x2000 + hart * 0x100
}
#[inline(always)]
pub const fn plic_senable(hart: u64) -> u64 {
    PLIC + 0x2080 + hart * 0x100
}
#[inline(always)]
pub const fn plic_mpriority(hart: u64) -> u64 {
    PLIC + 0x200000 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_spriority(hart: u64) -> u64 {
    PLIC + 0x201000 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_mclaim(hart: u64) -> u64 {
    PLIC + 0x200004 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_sclaim(hart: u64) -> u64 {
    PLIC + 0x201004 + hart * 0x2000
}

/// map the trampoline page to the highest address,
/// in both user and kernel space.
pub const TRAMPOLINE: u64 = MAXVA - PGSIZE;

/// map kernel stacks beneath the trampoline,
/// each surrounded by invalid guard pages.
#[inline(always)]
pub const fn kstack(p: u64) -> u64 {
    TRAMPOLINE - (p + 1) * 2 * PGSIZE
}

/// User memory layout.
/// Address zero first:
///   text
///   original data and bss
///   fixed-size stack
///   expandable heap
///   ...
///   TRAPFRAME (p->trapframe, used by the trampoline)
///   TRAMPOLINE (the same page as in the kernel)
pub const TRAPFRAME: u64 = TRAMPOLINE - PGSIZE;
