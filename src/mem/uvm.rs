use crate::{
    arch::{
        def::pgroundup,
        vm::{free_pagetable, map_pages, PageTable},
    },
    mem::alloc::{kalloc, kfree, page_size, LinkListAllocator},
};
use rv64::vm::{PteFlags, PTE};

pub struct UserPageTable(*mut PageTable);

impl UserPageTable {
    pub fn new() -> Option<UserPageTable> {
        let page = unsafe { kalloc(true) }?;
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

        let oldsz = pgroundup(oldsz);
        for a in (oldsz..newsz).step_by(pg_size) {
            if let Some(page) = unsafe { kalloc(true) } {
                unsafe {
                    if map_pages(a, pg_size, page.into(), perm.into()).is_err() {
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
}
