#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use os::{println};
use core::panic::PanicInfo;
use bootloader::{BootInfo,entry_point};
use alloc::{boxed::Box,vec,vec::Vec,rc::Rc};

entry_point!(kernel_main);


fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os::allocator;
    use os::memory::{self,BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    println!("HELLO WORLD {}","!");
    //initialize interrupts
    os::init();
    //mapper + frame allocator
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe{
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    //initialize heap mem
    allocator::init_heap(&mut mapper,&mut frame_allocator).expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}",heap_value);

    //create a dynamic sized vector
    let mut vec = Vec::new();
    for i in 0..500{
        vec.push(i);
    }
    println!("vec at {:p}",vec.as_slice());

    //create a referece counted vector -> will be freed when count reaches 0
    let referece_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = referece_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(referece_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    //tests
    #[cfg(test)]
    test_main();
    
    println!("IT DID NOT CRASH!");
    os::hlt_loop();
}



/// This function is called on panic.
#[cfg(not(test))] //use in cargo run 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}",info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion(){
    assert_eq!(1,1);
}