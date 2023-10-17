#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod gdt;
mod interrupts;
mod logger;
mod serial;

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
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    log::info!("Hello from kernel!");

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    x86_64::instructions::interrupts::int3();
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

    logger::init(raw_frame_buffer, framebuffer_info);
    serial::init();
    gdt::init();
    interrupts::init();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

pub(crate) trait Testable {
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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[cfg(test)]
pub(crate) fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        x86_64::instructions::hlt();
    }
}
