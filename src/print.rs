use crate::sbi;
use core::fmt::Write;

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.as_bytes() {
            sbi::putchar(*ch);
        }

        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    let mut console = Console;
    console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}
