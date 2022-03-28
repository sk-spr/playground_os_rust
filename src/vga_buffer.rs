#![feature(asm)]
use volatile::Volatile;
use spin::Mutex;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
///Color codes for VGA mode 3.
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
///Struct to represent a color code (foreground and background) for VGA mode 3 representation.
pub struct ColorCode(u8);
impl ColorCode{
    pub fn new(foreground:Color, background:Color) -> ColorCode{
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
///Struct representing a character with a color code and an ascii code for VGA mode 3 printing.
struct VGAChar{
    ascii_character : u8,
    color_code : ColorCode
}

///Height of VGA mode 3 buffer.
const BUFFER_WIDTH : usize = 80;
///Width of VGA mode 3 buffer.
const BUFFER_HEIGHT : usize = 25;

#[repr(transparent)]
///Struct representing the VGA mode 3 buffer.
pub struct Buffer{
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH];BUFFER_HEIGHT]
}

///Struct enabling writing to the specified VGA mode 3 buffer in the bottom line at the given column position, with the given colour code. Buffer may be specified but is usually located as 0xb8000 and represented as an unsafe dereferenced mutable pointer.
pub struct Writer{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer{
    ///Create a new Writer.
    pub fn new(col_pos: usize, col_code: ColorCode, buf : &'static mut Buffer) -> Writer{
        Writer{column_position: col_pos, color_code: col_code, buffer: buf}
    }
    ///Write a byte to the Buffer of the writer.
    pub fn write_byte(&mut self, byte:u8){
        match byte{
            b'\n' => self.new_line(),
            0xff => self.backspace(0),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(VGAChar{
                    ascii_character: byte,
                    color_code
                });
                self.column_position+=1;
            }
        }
    }
    ///Shift all lines in the buffer up, and clear the last line.
    fn new_line(&mut self){
        for y in 0..(BUFFER_HEIGHT - 1){
            for x in 0..(BUFFER_WIDTH){
                self.buffer.chars[y][x].write(self.buffer.chars[y+1][x].read());
                self.buffer.chars[y+1][x].write(VGAChar{
                    ascii_character: 0x0,
                    color_code: self.color_code
                });
            }
        }
        self.column_position = 0
    }
    ///Roll up the buffer and place column_position at the end of a line
    fn roll_up(&mut self){
        for y in (0..(BUFFER_HEIGHT-1)).rev(){
            for x in 0..BUFFER_WIDTH{
                self.buffer.chars[y+1][x].write(self.buffer.chars[y][x].read());
                self.buffer.chars[y][x].write(VGAChar{
                    ascii_character: 0x0,
                    color_code: self.color_code
                });
            }
        }
        self.column_position = BUFFER_WIDTH - 1;
    }

    ///Write a string to the buffer.
    pub fn write_string(&mut self, text : &str){
        for byte in text.bytes(){
            match byte{
                b'\n' | 0x20..=0x7e => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }
    ///Write a string with appended \n to the buffer.
    pub fn write_line(&mut self, text: &str){
        self.write_string(text);
        self.write_byte(b'\n');
    }
    ///Set writer's column position
    pub fn set_column_position(&mut self, pos: usize){
        self.column_position = pos;
    }
    ///Return writer's column position.
    pub fn get_column_position(&mut self) -> usize{
        self.column_position
    }
    pub fn backspace(&mut self, num_times_rolled_up : usize) {
        //inelegant hack
        let mut num_times_rolled_up_new = num_times_rolled_up;
        if self.column_position > 0{
            self.column_position -= 1;
        }else{
            num_times_rolled_up_new += 1;
            self.roll_up();
        }
        //skip null characters, stop if rolled over the whole screen
        let null_char = VGAChar{
            color_code: self.color_code,
            ascii_character: 0x0
        };
        let mut is_empty = false;
        if self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].read() == null_char{
            if num_times_rolled_up_new < 1 {
                self.backspace(num_times_rolled_up_new);
            } else {
                self.column_position = 0;
                is_empty = true;
                while self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].read() != null_char{
                    self.column_position += 1;
                }
            }
        }
        if !is_empty{
            self.write_byte(0x0);
            self.column_position -= 1;
        }

    }
}

const LONG_BUFFER_HEIGHT:usize = 1000 as usize;
struct LongBuffer{
    buffer: [[Volatile<VGAChar>; BUFFER_WIDTH];LONG_BUFFER_HEIGHT]
}
///Struct enabling double buffered writing to the VGA mode 3 buffer; enables multiple line backspacing, rolling up
pub struct BufferedWriter{
    writer: Writer,
    backup_buffer: LongBuffer
}
impl BufferedWriter{
    fn write_buf(&mut self){
        for i in 0..BUFFER_HEIGHT{
            //TODO: convert to longbuffer coords
            //TODO: write last part of longbuffer to writer
        }
    }
}
use core::fmt;
impl fmt::Write for Writer{
    ///Write a string using a writer. Complies with fmt::Write.
    fn write_str(&mut self, s:&str) -> fmt::Result{
        self.write_string(s);
        Ok(())
    }
}

//lazy static is used to avoid compiletime definition of static variables, making them initialize at runtime instead.
use lazy_static::lazy_static;
lazy_static!{
    ///Global writer, to be used by other functions.
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe{&mut *(0xb8000 as *mut Buffer)}
    });
}

pub fn init(){
    for y in 0..BUFFER_HEIGHT{
        for x in 0..BUFFER_WIDTH{
            WRITER.lock().write_byte(0x0);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).unwrap();
    })
}
//----------TEST CASES------------
#[test_case]
fn trivial_assertion(){
    assert_eq!(1,1);
}
#[test_case]
fn print_nopanic(){
    for i in 0..200{
        print!("A string abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!,;:dkgsoihoasdf");
    }
}
#[test_case]
fn println_nopanic(){
    for i in 0..200{
        println!("A line. Here is a line. Still going. Whatevs bruh.");
    }
}
#[test_case]
fn printed_exists(){
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    let s = "A string that fits within one line";
    interrupts::without_interrupts(||{
        let mut writer = WRITER.lock();
        writeln!(writer, "");
        write!(writer, "{}", s);
        for (i,c) in s.chars().enumerate(){
            assert_eq!(writer.buffer.chars[BUFFER_HEIGHT - 1][i].read().ascii_character, c as u8);
        }
    })
    
}