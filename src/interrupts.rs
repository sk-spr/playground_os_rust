use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{print, println, gdt, hlt_loop};
use spin;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::vga_buffer;
use crate::key_conversion::{KEYMAP_DE};
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
    //print!(".");
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
///Handle keyboard interrupts.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame){
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode : u8 = unsafe{port.read()};
    lazy_static!{
        static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> = spin::Mutex::new(Keyboard::new(layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore));
    }
    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode){
        if let Some(key) = keyboard.process_keyevent(key_event){
            match key{
                //TODO: write keyboard layout switching logic; global keymap var? files? todo.
                DecodedKey::Unicode(character) =>
                    match KEYMAP_DE.convert_char(character){
                        '\x08' => vga_buffer::WRITER.lock().backspace(0),
                        c => print!("{}", c)},
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
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



//------------TEST CASES--------------
#[test_case]
///Test whether breakpoint exceptions are caught.
fn test_breakpoint_exception(){
    x86_64::instructions::interrupts::int3();
}