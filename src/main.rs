#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use playground_os_rust::{allocator, memory, print, println, serial_print, serial_println};
use bootloader::{BootInfo, entry_point};
use alloc::boxed::Box;

static HELLO: &[u8] = b"Hello, world!";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
///Enum representing exit codes for QEMU debug shutdown device.
pub enum QemuExitCode{
    Success = 0x10,
    Failure = 0x11,
}
///Shutdown using QUEMU debug shutdown device (QEMU ONLY!)
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
#[cfg(test)]
fn test_runner(tests : &[&dyn Testable]){
    serial_println!("Running {} tests.", tests.len());
    for test in tests{
        test.run();
    }
    exit_qemu(QemuExitCode::Success)
}

entry_point!(kernel_main);

///Entry point for PlaygroundOS.
pub fn kernel_main(boot_info: &'static BootInfo) -> !{
    playground_os_rust::init(boot_info);
    serial_println!("Hello serial!");

    async fn async_fourty_two() -> u32{
        42
    }

    async fn example_task(){
        let number = async_fourty_two().await;
        println!("{}", number);
    }
    example_task();
    println!("Number of attached SCSI drives: {}",
             playground_os_rust::storage::scsi::check_number_of_scsi_drives());
    use playground_os_rust::pci::detect_pci_compat_bios;
    let has_pci = detect_pci_compat_bios();
    println!("System has PCI usable by configuration space 1: {}", has_pci);
    let pci_devices = *playground_os_rust::pci::check_all_pci_devices();
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_key_presses()));
    executor.run();
    playground_os_rust::hlt_loop();
}

use core::panic::PanicInfo;
use x86_64::VirtAddr;
use playground_os_rust::memory::translate_addr;
use playground_os_rust::task::simple_executor::SimpleExecutor;
use playground_os_rust::task::{keyboard, Task};
use playground_os_rust::task::executor::Executor;

#[cfg(not(test))]
#[panic_handler]
///Handle a panic.
fn panic(info: &PanicInfo) -> ! {
    println!("Panic!; {}", info);
    serial_println!("Panic!; {}", info);
    playground_os_rust::hlt_loop();
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failure);
    playground_os_rust::hlt_loop();
}


