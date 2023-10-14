#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod interrupts;
mod logger;
mod serial;

use crate::{interrupts::init_idt, logger::init_logger, serial::init_serial};
use bootloader_api::BootInfo;
use core::panic::PanicInfo;

bootloader_api::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {info}\n");
    exit_qemu(QemuExitCode::Failed);
    loop {
        x86_64::instructions::hlt();
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    log::info!("Hello from kernel!");

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    log::info!("It did not crash!");

    #[cfg(test)]
    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

fn init(boot_info: &'static mut BootInfo) {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    let framebuffer_info = framebuffer.info().clone();
    let raw_frame_buffer = framebuffer.buffer_mut();

    init_logger(raw_frame_buffer, framebuffer_info);
    init_serial();
    init_idt();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        let test_name = core::any::type_name::<T>();
        serial_print!("{test_name}...    ");
        self();
        serial_println!("[ok]");
    }
}

#[cfg(test)]
pub(crate) fn test_runner(tests: &[&dyn Testable]) {
    log::info!("running {} tests", tests.len());
    serial_println!("running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[cfg(test)]
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
