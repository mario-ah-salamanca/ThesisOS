#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use os::{println, task::{Task,simple_executor::SimpleExecutor},task::keyboard};
use core::panic::PanicInfo;
use bootloader::{BootInfo,entry_point};

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

    //keyboard

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses())); // new
    executor.run();

    
    //tests
    #[cfg(test)]
    test_main();
    
    println!("IT DID NOT CRASH!");
    os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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