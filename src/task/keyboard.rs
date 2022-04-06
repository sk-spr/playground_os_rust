use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use core::iter::Scan;
use futures_util::stream::{Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use futures_util::task::AtomicWaker;
use crate::key_conversion::KEYMAP_DE;
use crate::{println, vga_buffer};
use crate::print;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8){
    if let Ok(queue) = SCANCODE_QUEUE.try_get(){
        if let Err(_) = queue.push(scancode) {
            println!("warning: scancode queue full. dropping input.");
        } else{
            WAKER.wake(); //if a waker is registered, wake it. Otherwise, this is a nop
        }
    } else{
        println!("warning: queue uninitialised");
    }
}

pub struct ScancodeStream{
    _private: (),
}

impl ScancodeStream{
    pub fn new() -> ScancodeStream{
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once.");
        ScancodeStream{_private: ()}
    }
}
impl Stream for ScancodeStream{
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("cannot get queue, possibly uninitialised");

        //fast path
        if let Ok(scancode) = queue.pop(){
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop(){
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            },
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();

///Asynchronously print the queued key presses.
pub async fn print_key_presses(){
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore);
    while let Some(scancode) = scancodes.next().await{
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
    }

}