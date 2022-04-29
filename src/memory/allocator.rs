use alloc::alloc::{GlobalAlloc,Layout};
use core::{ptr::{null_mut}};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
#[path = "../allocators/bump.rs"] pub mod bump;
#[path = "../allocators/linked_list.rs"] pub mod linked_list;

//use bump::BumpAllocator;
use linked_list::LinkedListAllocator;
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100*1024; // 100Kib

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
)-> Result< () , MapToError<Size4KiB>>{
    //map all heap pages to physical frames
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range{
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe{
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }
    //exclusive reference to the wrapped HEAP
    unsafe{
        ALLOCATOR.lock().init(HEAP_START,HEAP_SIZE);
    }
    Ok(())
}

// A wrappaer around spin::Mutex to permit trait implementations

pub struct Locked<A>{
    inner: spin::Mutex<A>,
}

impl <A> Locked<A>{
    pub const fn new(inner:A) -> Self{
        Locked{
            inner: spin::Mutex::new(inner),
        }
    } 
    pub fn lock(&self) -> spin::MutexGuard<A>{
        self.inner.lock()
    }
}


//require align to be a power of 2, this is guarantee by GlobalAlloc
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// before refactoring
// fn align_up(addr:usize, align: usize) -> usize{
//     let remainder = addr % align;
//     if remainder == 0{
//         addr
//         addr - remainder * align
//     }
// }
pub struct Dummy;
//use linked_list_allocator::LockedHeap;
//implementations

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
// alloc_zeored and realloc methods have default implementations
// dont need to implemented again
#[global_allocator]
static ALLOCATOR:Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());