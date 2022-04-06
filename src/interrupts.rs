use core::arch::asm;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{print, println, gdt, hlt_loop};
use spin;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::instructions::port::Port;
use crate::vga_buffer;
use crate::key_conversion::{KEYMAP_DE};

pub static mut MILLISECONDS_ELAPSED: u64 = 0;
pub static PIT_MS_PER_INTERRUPT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
///Index in the PIC for various devices.
pub enum InterruptIndex{
    Timer = PIC_1_OFFSET,
    Keyboard,
}
// Helper functions for the Interrupt index.
impl InterruptIndex{
    fn as_u8(self)-> u8{
        self as u8
    }
    fn as_usize(self)->usize{
        usize::from(self.as_u8())
    }
}
lazy_static!{
    ///The global IDT.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe{
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}
//Handle PIC Timer interrupts.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame){
    unsafe{
        MILLISECONDS_ELAPSED += PIT_MS_PER_INTERRUPT as u64;
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
///Handle keyboard interrupts.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame){
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode : u8 = unsafe{port.read()};
    crate::task::keyboard::add_scancode(scancode);
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
///Initialize the Interrupt Descriptor Table.
pub fn init_idt(){
    IDT.load();
}
///Handler for breakpoint exceptions.
extern "x86-interrupt" fn breakpoint_handler(stack_frame:InterruptStackFrame){
    println!("Exception: Breakpoint \n {:#?}", stack_frame);
}
///Double fault handler.
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> !{
    panic!("Exception: Double Fault\n{:#?}", stack_frame);
}
use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode){
    use x86_64::registers::control::Cr2;
    println!("PAGE FAULT");
    println!("Accessed address: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

pub const PIC_1_OFFSET:u8 = 32;
pub const PIC_2_OFFSET:u8 = PIC_1_OFFSET + 8;

///Mutex struct representing the 8259 PICs 1 and 2.
pub static PICS:spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe{ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});

///Initialise the PIT to send an interrupt at the given frequency in Hz (times per second).
pub fn init_pit(frequency_in_hz: u32){
    //get the frequency in an allowed range
    let freq:u32 = if frequency_in_hz <= 18 {18}
        else if frequency_in_hz >= 1193181 {1193181}
        else {frequency_in_hz};
    //The port connected to the PIT
    let mut port: Port<u8> = Port::new(0x40);
    //a temporary value for storing the numerator of the division; needed as u32 would overflow
    let temp:u64 = (3579545 * 256 / 3 * 256);
    //reload value to be loaded into the PIT, calculated as per https://wiki.osdev.org/Pit
    let reload_value =  (temp / freq as u64) as u32;
    //split into bytes, to transmit the lower 2 bytes via the port
    let bytes = reload_value.to_ne_bytes();
    unsafe{
        port.write(bytes[2]);
        port.write(bytes[3]);
    }

}

//------------TEST CASES--------------
#[test_case]
///Test whether breakpoint exceptions are caught.
fn test_breakpoint_exception(){
    x86_64::instructions::interrupts::int3();
}