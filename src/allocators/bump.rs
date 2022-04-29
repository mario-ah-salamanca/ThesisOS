use super::{align_up,Locked};
use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr;
pub struct BumpAllocator{
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations:usize,
}

impl BumpAllocator{
    //create a new empty bumb allocator
    pub const fn new() -> Self{
        BumpAllocator{
            heap_start:0,
            heap_end:0,
            next:0,
            allocations:0,
        }
    }
    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self,heap_start:usize,heap_size:usize){
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start; //next = start, since the allocator is empty
    }
}


unsafe impl GlobalAlloc for Locked<BumpAllocator>{
    unsafe fn alloc(&self,layout:Layout) -> *mut u8{
        let mut bump = self.lock(); //get a mut ref
        //alignment and bounds check
        let alloc_start = align_up(bump.next,layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()){
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if alloc_end > bump.heap_end{
            ptr::null_mut() //out of mem
        }else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self,_ptr: *mut u8, _layout:Layout){
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0{
            bump.next = bump.heap_start;
        }
    }
}