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
mod terminal;
mod uninterruptible_mutex;

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

bootloader_api::entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    log::error!("{info}");

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    println!("[failed]\n");
    println!("Error: {info}\n");

    exit_qemu(QemuExitCode::Failed);
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    log::info!("Hello from kernel!");

    x86_64::instructions::interrupts::int3();

    log::info!("It did not crash!");

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        level_4_page_table.start_address()
    );

    // Note: The actual address might be different for you. Use the address that
    // your page fault handler reports.
    let ptr = 0x8000012b1b as *mut u8;

    // read from a code page
    unsafe {
        let x = *ptr;
        println!("x = {x}")
    }
    println!("read worked");

    // write to a code page
    unsafe {
        *ptr = 42;
    }
    println!("write worked");

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

    serial::init();
    terminal::init(raw_frame_buffer, framebuffer_info);
    logger::init();
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
        print!("{test_name}...    ");
        self();
        println!("[ok]");
    }
}

#[cfg(test)]
pub(crate) fn test_runner(tests: &[&dyn Testable]) {
    log::info!("running {} tests", tests.len());
    println!("running {} tests", tests.len());

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
