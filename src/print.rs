#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::uart::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! panic_print {
    ($($arg:tt)*) => ($crate::io::uart::panic(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! panic_println {
    () => ($crate::panic_print!("\n"));
    ($($arg:tt)*) => ($crate::panic_print!("{}\n", format_args!($($arg)*)));
}
