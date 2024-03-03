use int_enum::IntEnum;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum)]
#[repr(u8)]
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

/// Represents a color code that combines a foreground and background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Creates a new `ColorCode` with the specified foreground and background colors.
    ///
    /// # Arguments
    ///
    /// * `foreground` - The foreground color.
    /// * `background` - The background color.
    ///
    /// # Returns
    ///
    /// A new `ColorCode` instance.
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | foreground as u8)
    }

    /// Returns an empty `ColorCode`.
    ///
    /// # Returns
    ///
    /// An empty `ColorCode` instance.
    pub const fn empty() -> ColorCode {
        ColorCode(0)
    }

    /// Returns the foreground color of the `ColorCode`.
    ///
    /// # Returns
    ///
    /// The foreground color.
    pub fn foreground(&self) -> Color {
        Color::try_from(self.0).unwrap()
    }

    /// Returns the background color of the `ColorCode`.
    ///
    /// # Returns
    ///
    /// The background color.
    pub fn background(&self) -> Color {
        Color::try_from(self.0).unwrap()
    }
}

impl From<ColorCode> for u16 {
    fn from(color: ColorCode) -> u16 {
        color.0 as u16
    }
}

impl From<u8> for ColorCode {
    fn from(num: u8) -> ColorCode {
        ColorCode(num)
    }
}
