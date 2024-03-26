#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::uart::uart_print_sync(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! panic_print {
    ($($arg:tt)*) => ($crate::io::uart::uart_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! panic_println {
    () => ($crate::print_nosync!("\n"));
    ($($arg:tt)*) => ($crate::panic_print!("{}\n", format_args!($($arg)*)));
}
