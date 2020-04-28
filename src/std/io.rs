use core::fmt::Arguments;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::std::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    use core::fmt::Write;
    super::vga_buffer::WRITER.lock().write_fmt(args).unwrap();
}
