use bootloader_x86_64_common::framebuffer::FrameBufferWriter;
use core::fmt::{Arguments, Write};
use uart_16550::SerialPort;

use crate::interrupts::UninterruptibleMutex;

pub struct Terminal<'a> {
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
