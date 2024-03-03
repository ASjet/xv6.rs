use core::fmt;
use core::{cmp::max, fmt::Write};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use super::color::{Color, ColorCode};

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 25;
pub const INVALID_CHAR: u8 = 0xfe;

const BUFFER_ADDR: isize = 0xb8000;

type Register = u64;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> =
        Mutex::new(Writer::new(ColorCode::new(Color::White, Color::Black)));
}

pub fn set_color(color: ColorCode) {
    WRITER.lock().set_color(color);
}

pub fn get_color() -> ColorCode {
    WRITER.lock().get_color()
}

pub fn set_pos(row: usize, col: usize) {
    WRITER.lock().set_pos(row, col);
}

pub fn get_pos() -> (usize, usize) {
    WRITER.lock().get_pos()
}

pub fn clear() {
    WRITER.lock().clear();
}

#[macro_export]
macro_rules! with_color {
    ($color:expr, $block:expr) => {{
        let old_color = $crate::vga::get_color();
        $crate::vga::set_color($color);
        let result = $block;
        $crate::vga::set_color(old_color);
        result
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    character: u8,
    color: ColorCode,
}

#[allow(dead_code)]
impl Char {
    pub const fn new(character: u8, color: ColorCode) -> Char {
        Char { character, color }
    }

    pub const fn empty() -> Char {
        Char {
            character: 0,
            color: ColorCode::empty(),
        }
    }
}

impl From<Char> for u8 {
    fn from(ch: Char) -> u8 {
        ch.character
    }
}

impl From<Char> for ColorCode {
    fn from(ch: Char) -> ColorCode {
        ch.color
    }
}

#[repr(transparent)]
struct BufferLine([Volatile<Char>; BUFFER_WIDTH]);

impl BufferLine {
    pub fn write_line(&mut self, line: &BufferLine) {
        let dst = self.to_register_mut();
        line.to_register()
            .iter()
            .enumerate()
            .for_each(|(i, reg)| dst[i].write(reg.read()));
    }

    pub fn write_char(&mut self, index: usize, ch: Char) {
        self.0[index].write(ch);
    }

    pub fn read_char(&self, index: usize) -> Char {
        self.0[index].read()
    }

    pub fn clear(&mut self) {
        for reg in to_register_mut(&mut self.0) {
            // FIXME: Should the color be kept?
            reg.write(0);
        }
    }

    fn to_register(&self) -> &[Volatile<Register>] {
        to_register(&self.0)
    }

    fn to_register_mut(&mut self) -> &mut [Volatile<Register>] {
        to_register_mut(&mut self.0)
    }

    /// Compares the line with the specified string for test purposes.
    fn compare_str(&self, s: &str) -> Option<usize> {
        for (i, char) in s.bytes().enumerate() {
            match char {
                b'\n' => {
                    break;
                }
                _ if char == u8::from(self.read_char(i)) => {
                    continue;
                }
                _ => {
                    return Some(i);
                }
            }
        }
        None
    }
}

type Buffer = [BufferLine; BUFFER_HEIGHT];

pub struct Writer {
    row_pos: usize,
    col_pos: usize,
    color: ColorCode,
    buf: &'static mut Buffer,
}

#[allow(dead_code)]
impl Writer {
    /// Creates a new `Writer` with the specified `color`.
    ///
    /// # Arguments
    ///
    /// * `color` - The color code to set for the writer.
    ///
    /// # Returns
    ///
    /// A new `Writer` instance.
    pub fn new(color: ColorCode) -> Writer {
        Writer {
            row_pos: 0,
            col_pos: 0,
            color,
            buf: unsafe { &mut *(BUFFER_ADDR as *mut Buffer) },
        }
    }

    /// Sets the color code for the writer.
    ///
    /// # Arguments
    ///
    /// * `color` - The color code to set.
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    pub fn get_color(&self) -> ColorCode {
        self.color
    }

    /// Sets the position of the writer to the specified `row` and `col`.
    ///
    /// # Arguments
    ///
    /// * `row` - The row position to set.
    /// * `col` - The column position to set.
    ///
    /// # Notes
    ///
    /// The `row` must be less than `BUFFER_HEIGHT`.
    /// The `col` must be less than `BUFFER_WIDTH`.
    pub fn set_pos(&mut self, row: usize, col: usize) {
        self.row_pos = row % BUFFER_HEIGHT;
        self.col_pos = col % BUFFER_WIDTH;
    }

    pub fn get_pos(&self) -> (usize, usize) {
        (self.row_pos, self.col_pos)
    }

    /// Writes a single byte to the buffer.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to write.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.newline();
                }

                self.buf[self.row_pos].write_char(self.col_pos, Char::new(byte, self.color));
                self.col_pos += 1;
            }
        }
    }

    /// Writes a string to the buffer.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to write.
    pub fn write_str(&mut self, s: &str) {
        s.bytes()
            .map(convert_unprintable)
            .for_each(|b| self.write_byte(b));
    }

    /// Moves the writer to the next line.
    pub fn newline(&mut self) {
        self.row_pos = if self.row_pos < BUFFER_HEIGHT - 1 {
            self.row_pos.wrapping_add(1)
        } else {
            self.scroll(1);
            self.row_pos
        };
        self.col_pos = 0;
        // FIXME: Should the line here get cleared?
    }

    pub fn clear(&mut self) {
        self.buf.iter_mut().for_each(BufferLine::clear);
        self.row_pos = 0;
        self.col_pos = 0;
    }

    fn scroll(&mut self, count: usize) {
        if count >= self.row_pos {
            self.clear();
            self.row_pos = 0;
        } else {
            for row in count..BUFFER_HEIGHT {
                self.copy_line(row, row - count);
            }
            self.row_pos -= count;
        }

        self.buf[BUFFER_HEIGHT - count..BUFFER_HEIGHT]
            .iter_mut()
            .for_each(BufferLine::clear);
    }

    fn copy_line(&mut self, src: usize, dst: usize) -> bool {
        if src == dst || src >= BUFFER_HEIGHT || dst >= BUFFER_HEIGHT {
            return false;
        }

        let (left, right) = self.buf.split_at_mut(max(src, dst));

        let (src_line, dst_line) = if src > dst {
            (&right[0], &mut left[dst])
        } else {
            (&left[src], &mut right[0])
        };

        dst_line.write_line(src_line);

        true
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

fn convert_unprintable(byte: u8) -> u8 {
    match byte {
        0x20..=0x7e | b'\n' => byte,
        _ => INVALID_CHAR,
    }
}

// Convert a slice of T to a slice of register words to get higher read performance
fn to_register<T>(arr: &[T]) -> &[Volatile<Register>] {
    unsafe { &*((arr as *const [T]) as *const [Volatile<Register>]) }
}

// Convert a slice of T to a slice of register words to get higher write performance
fn to_register_mut<T>(arr: &mut [T]) -> &mut [Volatile<Register>] {
    unsafe { &mut *((arr as *mut [T]) as *mut [Volatile<Register>]) }
}

#[test_case]
fn test_write_byte() {
    let mut writer = WRITER.lock();
    writer.col_pos = 0;
    writer.row_pos = 0;
    writer.write_byte(42);
    assert_eq!(u8::from(writer.buf[0].read_char(0)), 42);
}

#[test_case]
fn test_set_pos() {
    let mut writer = WRITER.lock();
    writer.set_pos(5, 5);
    writer.write_byte(42);
    assert_eq!(u8::from(writer.buf[5].read_char(5)), 42);
}

#[test_case]
fn test_clear() {
    let mut writer = WRITER.lock();
    writer.set_pos(5, 5);
    writer.write_byte(42);
    assert_eq!(u8::from(writer.buf[5].read_char(5)), 42);
    writer.clear();
    assert_eq!(u8::from(writer.buf[5].read_char(5)), 0);
}

#[test_case]
fn test_write_str() {
    let mut writer = WRITER.lock();
    writer.clear();
    let s = "some output";
    writer.write_str(s);
    assert_eq!(writer.buf[0].compare_str(s), None);
}

#[test_case]
fn test_newline() {
    let mut writer = WRITER.lock();
    writer.clear();
    writer.newline();
    let s = "some output";
    assert_eq!(writer.buf[1].compare_str(s), Some(0));
    writer.write_str(s);
    assert_eq!(writer.buf[1].compare_str(s), None);
}

#[test_case]
fn test_write_multiline() {
    let mut writer = WRITER.lock();
    writer.clear();
    let s = "line1\nline2";
    writer.write_str(s);
    let mut iter = s.split_whitespace();
    assert_eq!(writer.buf[0].compare_str(iter.next().unwrap()), None);
    assert_eq!(writer.buf[1].compare_str(iter.next().unwrap()), None);
}
