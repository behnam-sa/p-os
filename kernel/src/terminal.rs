use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::framebuffer::FrameBufferWriter;
use conquer_once::spin::OnceCell;
use core::fmt::{Arguments, Write};
use klib::{
    interrupts::UninterruptibleMutex,
    io::{set_print_handler, Terminal},
};

use crate::serial::SERIAL1;

pub(crate) static FRAME_BUFFER_WRITER: OnceCell<UninterruptibleMutex<FrameBufferWriter>> =
    OnceCell::uninit();

pub(crate) static TERMINAL: UninterruptibleMutex<Terminal> =
    UninterruptibleMutex::new(Terminal::new(None, None));

pub(crate) fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) {
    let frame_buffer_writer = FRAME_BUFFER_WRITER
        .get_or_init(move || UninterruptibleMutex::new(FrameBufferWriter::new(framebuffer, info)));

    *TERMINAL.lock() = if cfg!(test) {
        Terminal::new(None, Some(&SERIAL1))
    } else {
        Terminal::new(Some(frame_buffer_writer), Some(&SERIAL1))
    };

    set_print_handler(&print_handler).unwrap();
}

#[doc(hidden)]
fn print_handler(args: Arguments) {
    TERMINAL.lock().write_fmt(args).unwrap();
}
