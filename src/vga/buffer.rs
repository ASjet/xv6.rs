use super::color::ColorCode;

const BUFFER_ADDR: isize = 0xb8000;
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const EMPTY_BUFFER: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT] =
    [[Char::empty(); BUFFER_WIDTH]; BUFFER_HEIGHT];

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
    chars: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    row_pos: usize,
    col_pos: usize,
    color: ColorCode,
    buf: &'static mut Buffer,
}

#[allow(dead_code)]
impl Writer {
    pub fn new(color: ColorCode) -> Writer {
        Writer {
            row_pos: 0,
            col_pos: 0,
            color,
            buf: unsafe { &mut *(BUFFER_ADDR as *mut Buffer) },
        }
    }

    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    pub fn set_pos(&mut self, row: usize, col: usize) {
        self.row_pos = row % BUFFER_HEIGHT;
        self.col_pos = col % BUFFER_WIDTH;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.newline();
                }

                self.buf.chars[self.row_pos][self.col_pos] = Char::new(byte, self.color);
                self.col_pos += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        s.bytes()
            .map(convert_unprintable)
            .for_each(|b| self.write_byte(b));
    }

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
                self.buf.chars[row - count] = self.buf.chars[row];
            }
            self.row_pos -= count;
        }
        for row in BUFFER_HEIGHT - count..BUFFER_HEIGHT {
            self.buf.chars[row] = EMPTY_BUFFER[0];
        }
    }

    fn clear(&mut self) {
        self.buf.chars.copy_from_slice(&EMPTY_BUFFER);
    }
}

fn convert_unprintable(byte: u8) -> u8 {
    match byte {
        0x20..=0x7e | b'\n' => byte,
        _ => 0xfe,
    }
}
