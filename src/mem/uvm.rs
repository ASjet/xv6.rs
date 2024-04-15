use crate::{
    arch::{
        def::{pgrounddown, pgroundup},
        vm::{free_pagetable, PageTable},
    },
    mem::alloc::{kalloc, kfree, page_size, LinkListAllocator, ALLOCATOR},
};
use core::ptr::addr_of;
use rv64::vm::{PageTableError, PteFlags, PTE};

pub struct UserPageTable(*mut PageTable);

impl UserPageTable {
    pub fn new() -> Option<UserPageTable> {
        let page = kalloc(true)?;
        Some(UserPageTable(page.as_mut_ptr::<PageTable>()))
    }

    /// Allocate PTEs and physical memory to grow process from oldsz to
    /// newsz, which need not be page aligned.  Returns new size or 0 on error.
    pub fn alloc(&mut self, oldsz: usize, newsz: usize) -> usize {
        if newsz < oldsz {
            return oldsz;
        }

        let pg_size = page_size();
        let perm = PteFlags::new()
            .set_writable(true)
            .set_executable(true)
            .set_readable(true)
            .set_user(true);
        let alloc = unsafe { &*addr_of!(ALLOCATOR) };

        let oldsz = pgroundup(oldsz);
        for a in (oldsz..newsz).step_by(pg_size) {
            if let Some(page) = kalloc(true) {
                unsafe {
                    if (*self.0).map_pages(a, pg_size, page, perm, alloc).is_err() {
                        kfree(page);
                        self.dealloc(a, oldsz);
                        return 0;
                    }
                }
            } else {
                self.dealloc(a, oldsz);
                return 0;
            }
        }
        newsz
    }

    /// Deallocate user pages to bring the process size from oldsz to
    /// newsz.  oldsz and newsz need not be page-aligned, nor does newsz
    /// need to be less than oldsz.  oldsz can be larger than the actual
    /// process size.  Returns the new process size.
    pub fn dealloc(&mut self, oldsz: usize, newsz: usize) -> usize {
        if newsz >= oldsz {
            return oldsz;
        }

        let oldsz_ru = pgroundup(oldsz);
        let newsz_ru = pgroundup(newsz);
        if newsz_ru < oldsz_ru {
            let npages = (oldsz_ru - newsz_ru) / page_size();
            unsafe {
                self.unmap(newsz_ru, npages, true);
            }
        }
        newsz
    }

    /// Remove npages of mappings starting from va. va must be
    /// page-aligned. The mappings must exist.
    /// Optionally free the physical memory.
    pub unsafe fn unmap(&mut self, va: usize, npages: usize, do_free: bool) {
        assert_eq!(va % page_size(), 0, "uvmunmap: not aligned");
        (va..va + npages * page_size())
            .step_by(page_size())
            .for_each(|a| {
                let (_, pte) = (*self.0)
                    .walk(a, 0, None::<&LinkListAllocator>)
                    .expect("uvmunmap: walk");
                let flags = pte.flags();
                assert!(flags.valid());
                assert!(flags.is_leaf());
                if do_free {
                    kfree(pte.addr());
                }
                *pte = PTE::new_invalid();
            });
    }

    pub unsafe fn free(&mut self, sz: usize) {
        if sz > 0 {
            self.unmap(0, pgroundup(sz) / page_size(), true);
        }
        free_pagetable(self.0);
    }

    /// mark a PTE invalid for user access.
    /// used by exec for the user stack guard page.
    /// Safety: va must be a valid virtual address.
    pub unsafe fn clear(&mut self, va: usize) {
        let (_, pte) =
            unsafe { (*self.0).walk(va, 0, None::<&LinkListAllocator>) }.expect("uvmclear: walk");
        *pte = PTE::new(pte.addr(), pte.flags().set_user(false));
    }

    /// Given a parent process's page table, copy
    /// its memory into a child's page table.
    /// Copies both the page table and the
    /// physical memory.
    /// returns 0 on success, -1 on failure.
    /// frees any allocated pages on failure.
    /// Safety: `sz` must be a valid size.
    pub unsafe fn clone(&self, sz: usize) -> Option<UserPageTable> {
        let mut new = UserPageTable::new()?;
        let pg_size = page_size();
        let alloc = &*addr_of!(ALLOCATOR);
        for a in (0..sz).step_by(pg_size) {
            let (_, pte) = unsafe { (*self.0).walk(a, 0, None::<&LinkListAllocator>) }
                .expect("UserPageTable::copy: pte should exist");
            let flags = pte.flags();
            assert!(flags.valid(), "UserPageTable::copy: page not present");

            let mem = kalloc(true).or_else(|| {
                new.unmap(0, a / pg_size, true);
                None
            })?;

            core::ptr::copy_nonoverlapping(
                pte.addr().as_ptr::<u8>(),
                mem.as_mut_ptr::<u8>(),
                pg_size,
            );

            (*new.0)
                .map_pages(a, pg_size, mem, flags, alloc)
                .ok()
                .or_else(|| {
                    kfree(mem);
                    new.unmap(0, a / pg_size, true);
                    None
                })?;
        }
        Some(new)
    }

    /// Copy from kernel to user.
    /// Copy len bytes from src to virtual address dstva in a given page table.
    /// Return 0 on success, -1 on error.
    pub unsafe fn copy_out(
        &self,
        dstva: usize,
        src: *const u8,
        len: usize,
    ) -> Result<(), UserPageTableError> {
        let mut len = len;
        let mut src = src;
        let mut dst = dstva;
        let pg_size = page_size();
        while len > 0 {
            let va0 = pgrounddown(dst);
            let (_, pte) = (*self.0)
                .walk(va0, 0, None::<&LinkListAllocator>)
                .map_err(|e| UserPageTableError::PageTableError(e))?;
            let n = core::cmp::min(pg_size - (dst - va0), len);
            core::ptr::copy_nonoverlapping(src, pte.addr().as_mut_ptr::<u8>().add(dst - va0), n);
            len -= n;
            src = src.add(n);
            dst = va0 + pg_size;
        }
        Ok(())
    }

    /// Copy from user to kernel.
    /// Copy len bytes to dst from virtual address srcva in a given page table.
    /// Return 0 on success, -1 on error.
    pub unsafe fn copy_in(
        &self,
        dst: *mut u8,
        srcva: usize,
        len: usize,
    ) -> Result<(), UserPageTableError> {
        let mut len = len;
        let mut src = srcva;
        let mut dst = dst;
        let pg_size = page_size();
        while len > 0 {
            let va0 = pgrounddown(src);
            let (_, pte) = (*self.0)
                .walk(va0, 0, None::<&LinkListAllocator>)
                .map_err(|e| UserPageTableError::PageTableError(e))?;
            let n = core::cmp::min(pg_size - (src - va0), len);
            core::ptr::copy_nonoverlapping(pte.addr().as_ptr::<u8>().add(src - va0), dst, n);
            len -= n;
            dst = dst.add(n);
            src = va0 + pg_size;
        }
        Ok(())
    }

    /// Copy a null-terminated string from user to kernel.
    /// Copy bytes to dst from virtual address srcva in a given page table,
    /// until a '\0', or max.
    /// Return 0 on success, -1 on error.
    pub unsafe fn copy_in_str(
        &self,
        dst: *mut u8,
        srcva: usize,
        max: usize,
    ) -> Result<(), UserPageTableError> {
        let mut got_null = false;
        let pg_size = page_size();
        let mut max = max;
        let mut srcva = srcva;
        while !got_null && max > 0 {
            let va0 = pgrounddown(srcva);
            let (_, pte) = (*self.0)
                .walk(va0, 0, None::<&LinkListAllocator>)
                .map_err(|e| UserPageTableError::PageTableError(e))?;
            let n = core::cmp::min(pg_size - (srcva - va0), max);
            let p = pte.addr().as_ptr::<u8>().add(srcva - va0);
            for i in 0..n {
                let ch = *p.add(i);
                if ch == 0 {
                    *dst.add(i) = 0;
                    got_null = true;
                    break;
                } else {
                    *dst.add(i) = *p.add(i);
                }
            }
            max -= n;
            srcva = va0 + page_size();
        }

        if got_null {
            Ok(())
        } else {
            Err(UserPageTableError::InvalidString)
        }
    }
}

pub enum UserPageTableError {
    PageTableError(PageTableError),
    InvalidString,
}
