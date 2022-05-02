#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(const_mut_refs)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
use core::panic::PanicInfo;

#[path ="modules/uart/serial.rs"]pub mod serial;
#[path ="modules/vga/vga_buffer.rs"]pub mod vga_buffer;
#[path = "interrupts/interrupts.rs"] pub mod interrupts;
#[path = "interrupts/gdt.rs"] pub mod gdt;
#[path = "memory/memory.rs"] pub mod memory;
#[path = "memory/allocator.rs"] pub mod allocator;
#[path = "task/mod.rs"] pub mod task;
extern crate alloc;


pub fn init(){
    gdt::init();
    interrupts::init_idt();
    unsafe{interrupts::PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop()->!{
    loop{
        x86_64::instructions::hlt();
    }
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// exit for qemu via a serial i/o input
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

//memory allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
/// Entry point for `cargo test`
#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}
// test panic handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}