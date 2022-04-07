#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
// set the name of the main test function
// it only compiles when call with cargo test
#![reexport_test_harness_main="test_main"]

#[path="modules/vga/vga_buffer.rs"] mod vga_buffer;
#[path="modules/uart/serial.rs"] mod serial;

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

/// This function is called on panic.
#[cfg(not(test))] //use in run mode 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}",info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    serial_println!("[failed]\n");
    serial_println!("Error:{}\n",info);
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

// this function is the entry point, since the linker looks for a function
// named `_start` by default
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("* step 1 dont be a hoe \n * step 2 enjoy your day");

    #[cfg(test)]
    test_main();

    loop{} 
}



#[derive(Debug,Clone,Copy, PartialEq, Eq)]
#[repr(u32)]

pub enum QemuExitCode{
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode){
    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

impl<T> Testable for T
where 
    T: Fn(),
{
    fn run(&self){
        serial_print!("{}...\t",core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
pub trait Testable{
    fn run(&self)->();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]){
    serial_println!("Running {} tests", tests.len());
    for test in tests{
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion(){
    assert_eq!(1,0);
}
