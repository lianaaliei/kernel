use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3, Red = 4, Magenta = 5,
    Brown = 6, LightGray = 7, DarkGray = 8, LightBlue = 9, LightGreen = 10,
    LightCyan = 11, LightRed = 12, Pink = 13, Yellow = 14, White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; WIDTH]; HEIGHT],
}

pub struct Writer {
    col: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= WIDTH {
                    self.new_line();
                }
                self.buffer.chars[HEIGHT - 1][self.col].write(ScreenChar {
                    ascii: byte,
                    color: self.color,
                });
                self.col += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }
        self.clear_row(HEIGHT - 1);
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar { ascii: b' ', color: self.color };
        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            match b {
                0x20..=0x7e | b'\n' => self.write_byte(b),
                _ => self.write_byte(0xfe),
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col: 0,
        color: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn clear() {
    let mut w = WRITER.lock();
    for row in 0..HEIGHT {
        w.clear_row(row);
    }
    w.col = 0;
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
