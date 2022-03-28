use alloc::alloc::{GlobalAlloc, Layout};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::{null, null_mut};
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    }, VirtAddr
};


pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100KiB

pub struct Dummy;
unsafe impl GlobalAlloc for Dummy{
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8{
        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called");
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>>{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = VirtAddr::new((HEAP_START + HEAP_SIZE - 1) as u64);
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range{
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
            ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        }
    }
    Ok(())
}

#[global_allocator]
static ALLOCATOR:LockedHeap = LockedHeap::empty();

#[test_case]
fn test_heap_box(){
    let b = Box::new(42);
}
#[test_case]
fn test_heap_vec(){
    let mut v = Vec::new();
    v.push(42);
    v.push(31);
    assert_eq!(v[0], 42);
    assert_eq!(v[1], 31);
}

