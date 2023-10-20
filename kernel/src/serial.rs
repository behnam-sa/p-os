use klib::interrupts::UninterruptibleMutex;
use uart_16550::SerialPort;

static SERIAL_PORT_BASE_NUMBER: u16 = 0x3F8;

pub(crate) static SERIAL1: UninterruptibleMutex<SerialPort> =
    UninterruptibleMutex::new(unsafe { SerialPort::new(SERIAL_PORT_BASE_NUMBER) });

pub(crate) fn init() {
    SERIAL1.lock().init();
}
