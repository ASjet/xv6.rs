use crate::println;

use crate::spinlock::Mutex;
use rv64::vm::{PhysAddr, PAGE_SIZE};

extern "C" {
    #[link_name = "_sheap"]
    static _heap_start: usize;
    #[link_name = "_heap_size"]
    static _heap_size: usize;
}

pub static mut ALLOCATOR: LinkListAllocator = LinkListAllocator::default();

pub fn init() {
    unsafe {
        // FIXME: panic on access _heap_start and _heap_size
        let start = PhysAddr::from(_heap_start);
        let end = PhysAddr::from(_heap_start + _heap_size);
        println!("init heap: {:?} - {:?}", start, end);
        ALLOCATOR = LinkListAllocator::new(start, end, PAGE_SIZE);
        ALLOCATOR.kfree_range(start, end);
    }
}

#[repr(C)]
pub struct FreePage {
    next: *mut FreePage,
}

impl FreePage {
    pub fn new(addr: PhysAddr) -> *mut Self {
        usize::from(addr) as *mut FreePage
    }
}

pub struct LinkListAllocator {
    heap_start: PhysAddr,
    heap_end: PhysAddr,
    page_size: usize,
    free_list: Mutex<*mut FreePage>,
}

impl LinkListAllocator {
    const fn default() -> Self {
        Self {
            heap_start: PhysAddr::null(),
            heap_end: PhysAddr::null(),
            page_size: 0,
            free_list: Mutex::new(0 as *mut FreePage, "free_list"),
        }
    }

    pub fn new(heap_start: PhysAddr, heap_end: PhysAddr, page_size: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            page_size,
            free_list: Mutex::new(0 as *mut FreePage, "free_list"),
        }
    }

    /// Allocate one 4096-byte page of physical memory.
    /// Returns a pointer that the kernel can use.
    /// Returns 0 if the memory cannot be allocated.
    pub unsafe fn kalloc(&self) -> Option<PhysAddr> {
        let mut free_list = self.free_list.lock();
        let page = *free_list;
        if page.is_null() {
            return None;
        }
        *free_list = (*page).next;

        let page = PhysAddr::from(page as usize);
        page.memset(5, self.page_size);
        Some(page)
    }

    /// Free the page of physical memory pointed at by v,
    /// which normally should have been returned by a
    /// call to kalloc().  (The exception is when
    /// initializing the allocator; see kinit above.)
    pub unsafe fn kfree(&self, page: PhysAddr) {
        if page.page_offset() != 0 || page < self.heap_start || page >= self.heap_end {
            panic!("dealloc: invalid page: {:?}", page);
        }

        unsafe {
            page.memset(1, self.page_size);
            let page = FreePage::new(page);
            let mut free_list = self.free_list.lock();
            (*page).next = *free_list;
            *free_list = page;
        }
    }

    pub unsafe fn kfree_range(&self, start: PhysAddr, end: PhysAddr) {
        for page in (usize::from(start.page_roundup())..=usize::from(end)).step_by(self.page_size) {
            self.kfree(PhysAddr::from(page));
        }
    }
}
