#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;


use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::boxed::Box;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use os::allocator;
    use os::memory::{self,BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe{ memory::init(phys_mem_offset)};
    let mut frame_allocator  = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap Initialization Failed!");
    test_main();
    loop{}
}

//test for basic allocation and allocation error
#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(50);
    let heap_value_2 = Box::new(20);
    assert_eq!(*heap_value_1 , 50);
    assert_eq!(*heap_value_2,20);
}

//test for large allocations and reallocs
use alloc::vec::Vec;
#[test_case]
fn large_vec(){
    let n = 1000;
    let mut large_boi = Vec::new();
    for i in 0..n{
        large_boi.push(i);
    }
    //nth partial sum to check that the sum was right
    assert_eq!(large_boi.iter().sum::<u64>(),(n-1)*n/2);
}

//multiple consecutive allocations
// checks that allocator reuses memory
use os::allocator::HEAP_SIZE;
#[test_case]
fn many_boxes(){
    for i in 0..HEAP_SIZE{
        let x = Box::new(i);
        assert_eq!(*x,i);
    }
}

//crashing the bump
#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1); // new
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1); // new
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}