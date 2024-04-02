#![allow(dead_code)]

pub const PG_SIZE: usize = 4096; // bytes per page
pub const PG_SHIFT: usize = 12; // bits of offset within a page

#[inline(always)]
pub const fn pgroundup(sz: usize) -> usize {
    (sz + PG_SIZE - 1) & !(PG_SIZE - 1)
}

#[inline(always)]
pub const fn pgrounddown(a: usize) -> usize {
    (a) & !(PG_SIZE - 1)
}

pub const PTE_V: usize = 1 << 0; // valid
pub const PTE_R: usize = 1 << 1;
pub const PTE_W: usize = 1 << 2;
pub const PTE_X: usize = 1 << 3;
pub const PTE_U: usize = 1 << 4; // 1 -> user can access

#[inline(always)]
pub const fn pa2pte(pa: usize) -> usize {
    ((pa) >> 12) << 10
}

#[inline(always)]
pub const fn pte2pa(pte: usize) -> usize {
    ((pte) >> 10) << 12
}

#[inline(always)]
pub const fn pte_flags(pte: usize) -> usize {
    (pte) & 0x3FF
}

pub const PX_MASK: usize = 0x1FF; // 9 bits

#[inline(always)]
pub const fn pxshift(level: usize) -> usize {
    PG_SHIFT + (9 * (level))
}

#[inline(always)]
pub const fn px(level: usize, va: usize) -> usize {
    ((va) >> pxshift(level)) & PX_MASK
}

pub const MAX_VA: usize = 1 << (9 + 9 + 9 + 12 - 1);

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
pub const KERNEL_BASE: usize = 0x80000000;
/// 128MB available RAM, same as in memory.x
pub const PHY_STOP: usize = KERNEL_BASE + 128 * 1024 * 1024;

/// qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x10000000;
pub const UART0_IRQ: usize = 10;

/// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_IRQ: usize = 1;

/// core local interruptor (CLINT), which contains the timer.
pub const CLINT: usize = 0x2000000;
#[inline(always)]
pub const fn clint_mtimecmp(hartid: usize) -> usize {
    CLINT + 0x4000 + 8 * hartid
}
pub const CLINT_MTIME: usize = CLINT + 0xBFF8; // cycles since boot.

/// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: usize = 0x0c000000;
pub const PLIC_PRIORITY: usize = PLIC + 0x0;
pub const PLIC_PENDING: usize = PLIC + 0x1000;

#[inline(always)]
pub const fn plic_menable(hart: usize) -> usize {
    PLIC + 0x2000 + hart * 0x100
}
#[inline(always)]
pub const fn plic_senable(hart: usize) -> usize {
    PLIC + 0x2080 + hart * 0x100
}
#[inline(always)]
pub const fn plic_mpriority(hart: usize) -> usize {
    PLIC + 0x200000 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_spriority(hart: usize) -> usize {
    PLIC + 0x201000 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_mclaim(hart: usize) -> usize {
    PLIC + 0x200004 + hart * 0x2000
}
#[inline(always)]
pub const fn plic_sclaim(hart: usize) -> usize {
    PLIC + 0x201004 + hart * 0x2000
}

/// map the trampoline page to the highest address,
/// in both user and kernel space.
pub const TRAMPOLINE: usize = MAX_VA - PG_SIZE;

/// map kernel stacks beneath the trampoline,
/// each surrounded by invalid guard pages.
#[inline(always)]
pub const fn kstack(p: usize) -> usize {
    TRAMPOLINE - (p + 1) * 2 * PG_SIZE
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
pub const TRAP_FRAME: usize = TRAMPOLINE - PG_SIZE;
