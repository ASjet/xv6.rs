use crate::arch;

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::arch::serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
