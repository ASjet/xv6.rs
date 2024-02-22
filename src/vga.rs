/// The VGA buffer module provides functionality for writing characters and strings to the VGA text mode buffer.
///
/// The VGA buffer is a 2-dimensional array of characters with a fixed width and height. Each character consists of a character code and a color code.
/// The `Writer` struct provides methods for writing characters and strings to the buffer, setting the color code, and moving the cursor position.
/// The `Buffer` struct represents the underlying buffer data structure.
/// The `Char` struct represents a single character in the buffer.
/// The `ColorCode` struct represents the color code for a character.
///
/// # Examples
///
/// ```
/// use super::buffer::{Writer, ColorCode};
///
/// // Create a new writer with a color code
/// let mut writer = Writer::new(ColorCode::new(Color::White, Color::Black));
///
/// // Write a string to the buffer and move to next line
/// writer.write_str("Hello, world!");
/// writer.newline();
///
/// // Same as above
/// writer.write_str("Hello, world!\n");
///
/// // Write a character to the buffer
/// writer.write_byte(b'A');
/// ```
///
/// # Notes
///
/// - The buffer width is fixed at 80 columns and the buffer height is fixed at 25 rows.
/// - The `set_pos` method wraps the row and column positions to fit within the buffer dimensions.
/// - The `newline` method moves the cursor to the next line and scrolls the buffer if necessary.
/// - The `scroll` method scrolls the buffer by the specified number of rows.
/// - The `clear` method clears the entire buffer.

mod color;
mod buffer;

pub use color::*;
pub use buffer::*;
