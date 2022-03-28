use x86_64::{PhysAddr, structures::paging::PageTable, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

///Provides the address of the currently active level 4 page table.
///
/// This function is unsafe because the caller must guarantee that the complete physical memory is mapped to virtual memory at the given offset. Also, this function must only be called once to avoid undefined behaviour arising from aliasing of &mut references.
unsafe fn active_level_4_table(physical_memory_offset : VirtAddr)->&'static mut PageTable{
    use x86_64::registers::control::Cr3;
    let (level_4_page_table_frame, _) = Cr3::read();
    let phys = level_4_page_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr : *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}
///Translates the given virtual address into the mapped physical address
///None if the address is unmapped
///
///This function is unsafe because the caller must guarantee that the complete
///physical memory is mapped to virtual memory at the passed offset
pub unsafe fn translate_addr(addr:VirtAddr, physical_memory_offset : VirtAddr)->Option<PhysAddr>{
    translate_addr_inner(addr, physical_memory_offset)
}
fn translate_addr_inner(addr:VirtAddr, offset:VirtAddr)->Option<PhysAddr>{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;
    let (level_4_page_frame, _) = Cr3::read();
    let table_indices = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_page_frame;
    for &index in &table_indices{
        //convert the frame into a page table reference
        let virt = offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe{&*table_ptr};
        //read the page table entry and update frame
        let entry = &table[index];
        frame = match entry.frame(){
            Ok(f) => f,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported")
        };
    }
    //translate the virtual address into a physical address
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
///Initialise the OffsetPageTable and return it.
pub unsafe fn init(physical_memory_offset : VirtAddr) -> OffsetPageTable<'static>{
    let level_4_page_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_page_table, physical_memory_offset)
}

///Testing function to create an example mapping.
pub fn create_example_mapping(
    page:Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
){
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe{
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

///A frame allocator that always returns "None"
pub struct EmptyFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}
///A frame allocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator{
    memory_map: &'static MemoryMap,
    next: usize,
}
impl BootInfoFrameAllocator{
    ///Create a frame allocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// passed memory map is valid. The main requirement is that all frames
    /// are marked as USABLE in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self{
        BootInfoFrameAllocator{
            memory_map,
            next: 0
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame>{
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r|r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr|PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}