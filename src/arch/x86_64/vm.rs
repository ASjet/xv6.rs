use bootloader::BootInfo;
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

static mut PHYSICAL_MEMORY_OFFSET: Option<usize> = None;

pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        PHYSICAL_MEMORY_OFFSET = Some(boot_info.physical_memory_offset as usize);
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
    inner_virt_to_phys(
        VirtAddr::new(virtual_address as u64),
        VirtAddr::new(PHYSICAL_MEMORY_OFFSET.unwrap() as u64),
    )
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
