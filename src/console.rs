use core::fmt::Write;

use crate::sbi;

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.as_bytes() {
            sbi::putchar(*ch);
        }

        Ok(())
    }
}
