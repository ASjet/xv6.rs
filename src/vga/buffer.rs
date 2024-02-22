use volatile::Volatile;

use super::color::ColorCode;

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 25;
pub const INVALID_CHAR: u8 = 0xfe;

const BUFFER_ADDR: isize = 0xb8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    character: u8,
    color: ColorCode,
}

impl Char {
    pub fn new(character: u8, color: ColorCode) -> Char {
        Char { character, color }
    }

    pub const fn empty() -> Char {
        Char {
            character: 0,
            color: ColorCode::empty(),
        }
    }
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

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

                self.buf.chars[self.row_pos][self.col_pos].write(Char::new(byte, self.color));
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
        for row in BUFFER_HEIGHT - count..BUFFER_HEIGHT {
            self.clear_line(row);
        }
    }

    fn copy_line(&mut self, src: usize, dest: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buf.chars[dest][col].write(self.buf.chars[src][col].read());
        }
    }

    fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_line(row);
        }
    }

    fn clear_line(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buf.chars[row][col].write(Char::empty());
        }
    }
}

fn convert_unprintable(byte: u8) -> u8 {
    match byte {
        0x20..=0x7e | b'\n' => byte,
        _ => INVALID_CHAR,
    }
}
