//! Early console via SBI — temporary until a userspace console server (0.5.x).

use core::fmt::{self, Write};

use crate::sbi;
use crate::sync::SpinLock;

static CONSOLE_LOCK: SpinLock = SpinLock::new();

struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        /* Expand \n → \r\n into a small kernel buffer, then bulk DBCN write. */
        let mut buf = [0u8; 256];
        let mut n = 0usize;
        for b in s.bytes() {
            if b == b'\n' {
                if n + 2 > buf.len() {
                    sbi::console_write(&buf[..n]);
                    n = 0;
                }
                buf[n] = b'\r';
                n += 1;
            }
            if n >= buf.len() {
                sbi::console_write(&buf[..n]);
                n = 0;
            }
            buf[n] = b;
            n += 1;
        }
        if n > 0 {
            sbi::console_write(&buf[..n]);
        }
        Ok(())
    }
}

/*
 * write_bytes - raw byte dump (no UTF-8 check); used by SYS_DEBUG_WRITE
 *
 * Copies through a kernel scratch so SBI DBCN sees a physical address.
 */
pub fn write_bytes(data: &[u8]) {
    let _g = CONSOLE_LOCK.lock();
    let mut buf = [0u8; 512];
    let mut n = 0usize;
    for &b in data {
        if b == b'\n' {
            if n + 2 > buf.len() {
                sbi::console_write(&buf[..n]);
                n = 0;
            }
            buf[n] = b'\r';
            n += 1;
        }
        if n >= buf.len() {
            sbi::console_write(&buf[..n]);
            n = 0;
        }
        buf[n] = b;
        n += 1;
    }
    if n > 0 {
        sbi::console_write(&buf[..n]);
    }
}

/*
 * _print - format to the early SBI console (SMP-safe)
 */
pub fn _print(args: fmt::Arguments) {
    let _g = CONSOLE_LOCK.lock();
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
