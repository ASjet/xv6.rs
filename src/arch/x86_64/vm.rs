use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::dmesg;

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

static mut PHYSICAL_MEMORY_OFFSET: Option<usize> = None;
static mut OFFSET_PAGE_TABLE: Option<OffsetPageTable<'static>> = None;
static mut FRAME_ALLOCATOR: Option<BootInfoFrameAllocator> = None;

/// Must init at early boot stage before any other function in this module
pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        PHYSICAL_MEMORY_OFFSET = Some(boot_info.physical_memory_offset as usize);
        dmesg!(
            "physical memory offset: {:#x}",
            PHYSICAL_MEMORY_OFFSET.unwrap()
        );
        OFFSET_PAGE_TABLE = Some(OffsetPageTable::new(
            load_page_table(cur_pgd_phyaddr()),
            VirtAddr::new(boot_info.physical_memory_offset),
        ));
        dmesg!("offset page table initialized");
        FRAME_ALLOCATOR = Some(BootInfoFrameAllocator::init(&boot_info.memory_map));
        init_heap(
            HEAP_START,
            HEAP_START + HEAP_SIZE,
            OFFSET_PAGE_TABLE.as_mut().unwrap(),
            FRAME_ALLOCATOR.as_mut().unwrap(),
        )
        .expect("heap initialization failed");
        dmesg!("heap initialized")
    }
}

pub unsafe fn load_page_table(physical_address: usize) -> &'static mut PageTable {
    &mut *(phys_to_virt(physical_address) as *mut PageTable)
}

pub unsafe fn cur_pgd_phyaddr() -> usize {
    cur_pgd().start_address().as_u64() as usize
}

pub unsafe fn phys_to_virt(physical_address: usize) -> usize {
    inner_phys_to_virt(
        PhysAddr::new(physical_address as u64),
        VirtAddr::new(PHYSICAL_MEMORY_OFFSET.unwrap() as u64),
    )
    .as_u64() as usize
}

pub unsafe fn virt_to_phys(virtual_address: usize) -> Option<usize> {
    OFFSET_PAGE_TABLE
        .as_ref()
        .unwrap()
        .translate_addr(VirtAddr::new(virtual_address as u64))
        .map(|addr| addr.as_u64() as usize)
}

fn cur_pgd() -> PhysFrame {
    use x86_64::registers::control::Cr3;
    let (l4_table_frame, _) = Cr3::read();
    l4_table_frame
}

fn inner_phys_to_virt(physical_address: PhysAddr, physical_memory_offset: VirtAddr) -> VirtAddr {
    physical_memory_offset + physical_address.as_u64()
}

#[allow(dead_code)]
fn inner_virt_to_phys(
    virtual_address: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;

    let mut frame = cur_pgd();

    let table_indexes = [
        virtual_address.p4_index(),
        virtual_address.p3_index(),
        virtual_address.p2_index(),
        virtual_address.p1_index(),
    ];

    for &index in &table_indexes {
        let virt = inner_phys_to_virt(frame.start_address(), physical_memory_offset);
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => {
                todo!("huge pages");
            }
        };
    }

    Some(frame.start_address() + u64::from(virtual_address.page_offset()))
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(PhysAddr::new)
            .map(PhysFrame::containing_address)
        // We can not save the iterator here cause it's just simply not supported yet
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn map_virt_to_phys(virtual_address: usize, physical_address: usize) -> Option<usize> {
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_address as u64));
    let frame = PhysFrame::containing_address(PhysAddr::new(physical_address as u64));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let align_virt_address = page.start_address().as_u64() as usize;
    unsafe {
        OFFSET_PAGE_TABLE.as_mut().unwrap().map_to(
            page,
            frame,
            flags,
            FRAME_ALLOCATOR.as_mut().unwrap(),
        )
    }
    .ok()?
    .flush();
    Some(align_virt_address)
}

// Map pages within the heap range to physical frames
pub fn init_heap(
    heap_start: usize,
    heap_end: usize,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let start_page = Page::containing_address(VirtAddr::new(heap_start as u64));
        let end_page = Page::containing_address(VirtAddr::new(heap_end as u64 - 1u64));
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    Ok(())
}
