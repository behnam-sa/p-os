#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod allocator;
mod gdt;
mod interrupts;
mod logger;
mod memory;
mod serial;
mod terminal;

use alloc::vec;
use alloc::{boxed::Box, rc::Rc, vec::Vec};
use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use klib::io::{print, println};
use x86_64::VirtAddr;

use crate::memory::BootInfoFrameAllocator;

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
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
bootloader_api::entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

#[allow(dead_code)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    {
        // create a dynamically sized vector
        let mut vec = Vec::new();
        for i in 0..500 {
            vec.push(i);
        }
        println!("vec at {:p}", vec.as_slice());
    }

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    // […] call `test_main` in test context
    println!("It did not crash!");

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    test_main();

    exit_qemu(QemuExitCode::Success);
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

    let physical_memory_offset =
        VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());

    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
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
    let test_count = tests.len();
    println!("\nrunning {test_count} tests\n");

    for test in tests {
        test.run();
    }

    println!("\ntest result: ok. {test_count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in ?.??s");
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
