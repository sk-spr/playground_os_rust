#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(playground_os_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use playground_os_rust::print;
#[no_mangle]
pub fn _start() -> !{
    test_main();
    loop{}
}
fn test_runner(tests: &[&dyn Fn()]){
    //TODO: implement test_runner
}
#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    playground_os_rust::test_panic_handler(info);
}
#[test_case]
fn print_test(){
    print!("Test");
}