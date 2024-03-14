#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::vga_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::arch::serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dmesg {
    ($($arg:tt)*) => ({
        #[cfg(not(test))]
        $crate::println!("[{:12.6}] {}", $crate::arch::interrupts::ticks(), format_args!($($arg)*));

        #[cfg(test)]
        $crate::serial_println!("[{:12.6}] {}", $crate::arch::interrupts::ticks(), format_args!($($arg)*));
    });
    () => ();
}