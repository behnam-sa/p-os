use core::fmt::{Arguments, Write};
use uart_16550::SerialPort;

use crate::uninterruptible_mutex::UninterruptibleMutex;

static SERIAL_PORT_BASE_NUMBER: u16 = 0x3F8;

pub(crate) static SERIAL1: UninterruptibleMutex<SerialPort> =
    UninterruptibleMutex::new(unsafe { SerialPort::new(SERIAL_PORT_BASE_NUMBER) });

pub(crate) fn init() {
    SERIAL1.lock().init();
}

#[doc(hidden)]
pub(crate) fn _print(args: Arguments) {
    SERIAL1.lock().write_fmt(args).unwrap();
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!("{}\n", format_args!($($arg)*)));
    }
}
