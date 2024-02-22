use core::fmt::{Arguments, Write};
use super::buffer::WRITER;
use super::ColorCode;

#[doc(hidden)]
pub fn _print(args: Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn set_color(color: ColorCode) {
    WRITER.lock().set_color(color);
}

pub fn set_pos(row: usize, col: usize) {
    WRITER.lock().set_pos(row, col);
}

pub fn clear() {
    WRITER.lock().clear();
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
