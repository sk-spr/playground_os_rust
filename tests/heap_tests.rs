#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(playground_os_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use playground_os_rust::allocator::HEAP_SIZE;

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> !{
    playground_os_rust::init(boot_info);
    test_main();
    playground_os_rust::hlt_loop();
}
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> !{
    playground_os_rust::test_panic_handler(info);
}

#[test_case]
fn test_alloc_box(){
    let heap_val_1 = Box::new(1);
    let heap_val_2 = Box::new(2);
    assert_eq!(*heap_val_1, 1);
    assert_eq!(*heap_val_2, 2);
}
#[test_case]
fn test_large_vec(){
    let mut vec = Vec::new();
    let upperbound = 1000;
    for i in 0..1000{
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (upperbound - 1) * upperbound / 2);
}
#[test_case]
fn many_boxes(){
    for i in 0..HEAP_SIZE{
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}