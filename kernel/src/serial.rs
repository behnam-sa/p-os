use spin::Mutex;
use uart_16550::SerialPort;

pub(crate) static SERIAL1: Mutex<SerialPort> = {
    let serial_port = unsafe { SerialPort::new(0x3F8) };
    Mutex::new(serial_port)
};

pub(crate) fn init_serial() {
    SERIAL1.lock().init();
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
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
