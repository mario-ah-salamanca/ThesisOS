
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
// (color_name,number)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color{
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

// ColorCode for a char
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]

struct ColorCode(u8);

impl ColorCode{
    fn new(foreground:Color,background:Color) -> ColorCode{
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// VGA compliant char
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar{
    ascii_char: u8,
    color_code: ColorCode,
}

// creating Buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer{
    characters: [[Volatile<ScreenChar>;BUFFER_WIDTH];BUFFER_HEIGHT],
}

pub struct Writer{
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer{
    pub fn write_byte(&mut self, byte:u8){
        match byte{
            b'\n' => self.new_line(),
            byte =>{
                // new line at the end of "terminal"
                if self.column_position >= BUFFER_WIDTH{
                    self.new_line()
                }
                // set the pointer at the needed position
                // save nessesary data
                let row:usize = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code:ColorCode = self.color_code;
                // write char to screen
                self.buffer.characters[row][col].write(ScreenChar{
                    ascii_char:byte,
                    color_code
                });
                //move pointer after write
                self.column_position +=1;
            }
        }
    }

    pub fn write_string(&mut self,s:&str){
        for byte in s.bytes(){
            match byte{
                // printable ascii byte or new line 
                // is going to be written
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _=> self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self){
        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH{
                let character = self.buffer.characters[row][col].read();
                self.buffer.characters[row-1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self,row:usize){
        let blank = ScreenChar{
            ascii_char:b' ',
            color_code:self.color_code,
        };
        for col in 0..BUFFER_WIDTH{
            self.buffer.characters[row][col].write(blank);
        }
    }
}

//for formatted strings
impl fmt::Write for Writer{
    fn write_str(&mut self,s:&str) -> fmt::Result{
        self.write_string(s);
        Ok(())
    }
}


// Global Interface
lazy_static!{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


// for making them available over the whole crate
// places the macro in the root namespace

// use crate::print or use crate::println
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
pub fn _print(args: fmt::Arguments){
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}



#[test_case]
fn test_println_simple(){
    println!("testprint simple output");
}
#[test_case]
fn test_println_many(){
    for _ in 0..200{
        println!("test_println_many output");
    }
}



#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(||{
        let mut writer = WRITER.lock();
        writeln!(writer ,"\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.characters[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });
}

#[test_case]
fn test_really_long_line(){
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    let s:&str = "this is goign to be a really long string to test if the VGA buffer is working properly and if it can print a long that takes a multiple lines inside the buffer. ----------------------------------";
    let len:usize = s.len();
    let mut nlin = 1;

    if len > BUFFER_WIDTH{
        nlin = len / BUFFER_WIDTH + 1;
    }
    interrupts::without_interrupts(||{
        let mut writer = WRITER.lock();
        writeln!(writer,"\n{}",s).expect("writeln failed");
        let mut rw_start:usize= BUFFER_HEIGHT- 2 - nlin;
        let mut current_char:usize = 0;
        for (i,c) in s.chars().enumerate(){
            if i%BUFFER_WIDTH == 0{
                current_char = 0;
                rw_start += 1;
            }
            let screen_char:ScreenChar = writer.buffer.characters[rw_start][current_char].read();
            assert_eq!(char::from(screen_char.ascii_char),c);  
            current_char += 1; 
        }
    });
}

