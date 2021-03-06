#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
extern crate alloc;
pub mod task;
pub mod key_conversion;
pub mod storage;
use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

#[cfg(test)]
entry_point!(test_kernel_main);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
///Enum representing exit codes for QEMU debug shutdown device.
pub enum QemuExitCode{
    Success = 0x10,
    Failure = 0x11,
}
///Shutdown using QEMU debug shutdown device (QEMU ONLY!)
pub fn exit_qemu(exit_code : QemuExitCode){
    use x86_64::instructions::port::Port;
    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
pub trait Testable{
    fn run(&self) -> ();
}
impl<T> Testable for T
where
    T:Fn(),
{
    fn run(&self){
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
pub fn test_runner(tests: &[&dyn Testable]){
    serial_println!("Running {} tests.", tests.len());
    for test in tests{
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}
pub fn test_panic_handler(info : &PanicInfo) -> !{
    serial_println!("[failed]; Error: {}", info);
    exit_qemu(QemuExitCode::Failure);
    hlt_loop();
}
pub fn hlt_loop() -> !{
    //loop{}
    loop{x86_64::instructions::hlt()}
}
pub fn init(boot_info: &'static bootloader::BootInfo){
    vga_buffer::init();
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pit((1 / interrupts::PIT_MS_PER_INTERRUPT) * 1000);
    unsafe{interrupts::PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();
    let phys_mem_off = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe{memory::init(phys_mem_off)};
    let mut frame_allocator = unsafe{memory::BootInfoFrameAllocator::init(&boot_info.memory_map)};
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialisation failed");
}
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> !{
    panic!("Allocation error: {:#?}", layout);
}

#[cfg(test)]
pub fn test_kernel_main(boot_info: &'static BootInfo) -> !{
    init(boot_info);
    test_main();
    hlt_loop();
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    test_panic_handler(info)
}