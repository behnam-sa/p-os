use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::framebuffer::FrameBufferWriter;
use conquer_once::spin::OnceCell;
use core::fmt::Arguments;
use core::fmt::Write;
use uart_16550::SerialPort;

use crate::serial::SERIAL1;
use crate::uninterruptible_mutex::UninterruptibleMutex;

pub(crate) struct Terminal<'a> {
    framebuffer: Option<&'a UninterruptibleMutex<FrameBufferWriter>>,
    serial_port: Option<&'a UninterruptibleMutex<SerialPort>>,
}

impl<'a> Terminal<'a> {
    pub const fn new(
        framebuffer: Option<&'a UninterruptibleMutex<FrameBufferWriter>>,
        serial_port: Option<&'a UninterruptibleMutex<SerialPort>>,
    ) -> Self {
        Self {
            framebuffer,
            serial_port,
        }
    }

    /// Force-unlock the logger to prevent a deadlock
    pub unsafe fn force_unlock(&self) {
        if let Some(framebuffer) = self.framebuffer {
            unsafe { framebuffer.force_unlock() };
        }
        if let Some(serial) = self.serial_port {
            unsafe { serial.force_unlock() };
        }
    }

    pub fn flush(&self) {}
}

impl Write for Terminal<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(framebuffer) = self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            framebuffer.write_str(s)?;
        };

        if let Some(serial_port) = self.serial_port {
            let mut serial_port = serial_port.lock();
            serial_port.write_str(s)?;
        };

        Ok(())
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> core::fmt::Result {
        if let Some(framebuffer) = self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            framebuffer.write_fmt(args)?;
        };

        if let Some(serial_port) = self.serial_port {
            let mut serial_port = serial_port.lock();
            serial_port.write_fmt(args)?;
        }

        Ok(())
    }
}

pub(crate) static FRAME_BUFFER_WRITER: OnceCell<UninterruptibleMutex<FrameBufferWriter>> =
    OnceCell::uninit();

pub(crate) static TERMINAL: UninterruptibleMutex<Terminal> =
    UninterruptibleMutex::new(Terminal::new(None, None));

pub(crate) fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) {
    let frame_buffer_writer = FRAME_BUFFER_WRITER
        .get_or_init(move || UninterruptibleMutex::new(FrameBufferWriter::new(framebuffer, info)));

    *TERMINAL.lock() = create_terminal(frame_buffer_writer);
}

#[cfg(not(test))]
fn create_terminal(frame_buffer_writer: &UninterruptibleMutex<FrameBufferWriter>) -> Terminal<'_> {
    Terminal::new(Some(frame_buffer_writer), Some(&SERIAL1))
}

#[cfg(test)]
fn create_terminal(frame_buffer_writer: &UninterruptibleMutex<FrameBufferWriter>) -> Terminal<'_> {
    Terminal::new(None, Some(&SERIAL1))
}

#[doc(hidden)]
pub(crate) fn _print(args: Arguments) {
    TERMINAL.lock().write_fmt(args).unwrap();
}

/// Prints to the terminal output
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::terminal::_print(format_args!($($arg)*))
    };
}

/// Prints to the terminal output appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::terminal::_print(format_args!("{}\n", format_args!($($arg)*)))
    }
}
