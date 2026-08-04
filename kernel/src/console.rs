//! Early console via SBI — temporary until a userspace console server (0.5.x).

use core::fmt::{self, Write};

use crate::sbi;

struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                sbi::console_putchar(b'\r');
            }
            sbi::console_putchar(b);
        }
        Ok(())
    }
}

/*
 * _print - format to the early SBI console
 *
 * Analogous to Linux early printk plumbing: single-hart, no lock yet.
 */
pub fn _print(args: fmt::Arguments) {
    let _ = Uart.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!("{}\n", format_args!($($arg)*)));
    }};
}
