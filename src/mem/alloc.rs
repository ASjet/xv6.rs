use crate::arch;
use crate::println;
use crate::spinlock::Mutex;
use rv64::vm::PageAllocator;
use rv64::vm::PageWidth;
use rv64::vm::{PhysAddr, PAGE_SIZE};

pub static mut ALLOCATOR: LinkListAllocator = LinkListAllocator::default();

#[inline]
pub const fn page_size() -> usize {
    unsafe { ALLOCATOR.page_size() }
}

#[inline]
pub fn free_pages() -> usize {
    unsafe { ALLOCATOR.free_pages() }
}

#[inline]
pub fn kalloc(zeroed: bool) -> Option<PhysAddr> {
    unsafe { ALLOCATOR.kalloc(zeroed) }
}

#[inline]
pub unsafe fn kfree(addr: impl Into<PhysAddr>) {
    ALLOCATOR.kfree(addr);
}

#[inline]
pub unsafe fn kfree_range(start: impl Into<PhysAddr>, end: impl Into<PhysAddr>) {
    ALLOCATOR.kfree_range(start, end);
}

pub fn init_heap() {
    let start = arch::vm::heap_start();
    let end = arch::vm::heap_end();
    println!("init heap: 0x{:x} - 0x{:x}", start, end);
    let pages = unsafe {
        ALLOCATOR = LinkListAllocator::new(start, end, PAGE_SIZE);
        ALLOCATOR.kfree_range(start, end);
        ALLOCATOR.free_pages()
    };
    println!(
        "{} pages ({} KiB) available",
        pages,
        pages * PAGE_SIZE / 1024
    );
}

#[repr(C)]
pub struct FreePage {
    next: *mut FreePage,
}

impl FreePage {
    pub fn new(addr: impl Into<usize>) -> *mut Self {
        addr.into() as *mut FreePage
    }
}

pub struct LinkListAllocator {
    heap_start: PhysAddr,
    heap_end: PhysAddr,
    page_size: usize,
    free_pages: Mutex<usize>,
    free_list: Mutex<*mut FreePage>,
}

impl LinkListAllocator {
    const fn default() -> Self {
        Self {
            heap_start: PhysAddr::null(),
            heap_end: PhysAddr::null(),
            page_size: 0,
            free_pages: Mutex::new(0, "free_pages"),
            free_list: Mutex::new(0 as *mut FreePage, "free_list"),
        }
    }

    pub fn new(
        heap_start: impl Into<PhysAddr>,
        heap_end: impl Into<PhysAddr>,
        page_size: usize,
    ) -> Self {
        Self {
            heap_start: heap_start.into(),
            heap_end: heap_end.into(),
            page_size,
            free_pages: Mutex::new(0, "init_free_pages"),
            free_list: Mutex::new(0 as *mut FreePage, "init_free_list"),
        }
    }

    pub const fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn free_pages(&self) -> usize {
        *self.free_pages.lock()
    }

    /// Allocate one 4096-byte page of physical memory.
    /// Returns a pointer that the kernel can use.
    /// Returns 0 if the memory cannot be allocated.
    pub unsafe fn kalloc(&self, zeroed: bool) -> Option<PhysAddr> {
        let mut free_list = self.free_list.lock();
        let page = *free_list;
        if page.is_null() {
            return None;
        }
        *free_list = (*page).next;
        *self.free_pages.lock() -= 1;

        let page = PhysAddr::from(page as usize);
        page.memset(
            if zeroed {
                0usize
            } else {
                0xAAAA_AAAA_AAAA_AAAA_usize
            },
            self.page_size,
        );

        Some(page)
    }

    /// Free the page of physical memory pointed at by v,
    /// which normally should have been returned by a
    /// call to kalloc().  (The exception is when
    /// initializing the allocator; see kinit above.)
    pub unsafe fn kfree(&self, addr: impl Into<PhysAddr>) {
        let page = addr.into();
        if page.page_offset() != 0 || page < self.heap_start || page > self.heap_end {
            panic!("kfree: invalid page: {:?}", page);
        }

        unsafe {
            page.memset(0xFFFF_FFFF_FFFF_FFFF_usize, self.page_size);
            let page = FreePage::new(page);
            let mut free_list = self.free_list.lock();
            (*page).next = *free_list;
            *free_list = page;
            *self.free_pages.lock() += 1;
        }
    }

    pub unsafe fn kfree_range(&self, start: impl Into<PhysAddr>, end: impl Into<PhysAddr>) {
        let start = start.into();
        let end = end.into();
        for page in (usize::from(start.page_roundup())..=usize::from(end)).step_by(self.page_size) {
            self.kfree(PhysAddr::from(page));
        }
    }
}

unsafe impl PageAllocator for LinkListAllocator {
    unsafe fn palloc(&self, page_width: PageWidth) -> Option<PhysAddr> {
        if page_width != PageWidth::W4K {
            return None;
        }
        self.kalloc(true)
    }

    unsafe fn pfree(&self, page: PhysAddr) {
        self.kfree(page)
    }
}

unsafe impl Sync for LinkListAllocator {}
unsafe impl Send for LinkListAllocator {}
