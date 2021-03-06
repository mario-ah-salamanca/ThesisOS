use x86_64::{
    VirtAddr,
    PhysAddr,
    structures::paging::{PageTable, PhysFrame, Size4KiB, FrameAllocator, OffsetPageTable}
};

use bootloader::{bootinfo::{MemoryMap,MemoryRegionType}};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator{
    memory_map: &'static MemoryMap,
    next:usize,
}


impl BootInfoFrameAllocator{
    pub unsafe fn init(memory_map: &'static MemoryMap)-> Self{
        BootInfoFrameAllocator{
            memory_map,
            next:0,
        }
    }

    fn usable_frames(&self)-> impl Iterator<Item = PhysFrame>{
        //get usable regions from the memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // creates 'PhysFrame' types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator{
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static>{
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}


unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame,_) = Cr3::read();

    let pa = level_4_table_frame.start_address();

    let va = physical_memory_offset + pa.as_u64();
    
    let page_table_ptr: *mut PageTable = va.as_mut_ptr();

    &mut * page_table_ptr //unsafe
}



pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator{
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}
