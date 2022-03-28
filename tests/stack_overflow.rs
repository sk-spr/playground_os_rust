#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use playground_os_rust::serial_print;

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    ///Testing Interrupt Descriptor Table.
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(playground_os_rust::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}
use playground_os_rust::{exit_qemu, QemuExitCode, serial_println};
use x86_64::structures::idt::InterruptStackFrame;

///Testing double fault handler, prints OK if a double fault is caught.
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

pub fn init_test_idt() {
    TEST_IDT.load();
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    playground_os_rust::gdt::init();
    init_test_idt();
    stack_overflow();
    panic!("Execution continued after Stack Overflow");
}
#[allow(unconditional_recursion)]
///Cause a stack overflow.
fn stack_overflow(){
    stack_overflow();
    volatile::Volatile::new(0).read();//prevent optimisations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    playground_os_rust::test_panic_handler(info)
}