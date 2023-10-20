#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

use std::{process::Command, time::Duration};

use terminal_size::terminal_size;
use wait_timeout::ChildExt;

fn main() {
    run_qemu(true, false);
}

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {
    run_qemu(true, true);
}

fn run_qemu(serial_output: bool, hide_window: bool) {
    let uefi_image = env!("UEFI_IMAGE");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file={uefi_image}"));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    if hide_window {
        qemu.arg("-display").arg("none");
    }

    if serial_output {
        qemu.arg("-device")
            .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
        qemu.arg("-serial").arg("stdio");

        if let Some(terminal_size) = terminal_size() {
            let (_, height) = terminal_size;

            for _ in 0..height.0 {
                println!();
            }
        }
    }

    let mut child = qemu.spawn().unwrap();

    let exit_status = if cfg!(test) {
        let test_timeout = Duration::from_secs(300);

        match child.wait_timeout(test_timeout).unwrap() {
            Some(status) => status,
            None => {
                // child hasn't exited yet
                child.kill().unwrap();
                child.wait().unwrap()
            }
        }
    } else {
        child.wait().unwrap()
    };

    let exit_code = exit_status.code().unwrap_or(-1);
    println!("\nQEMU exited with code {exit_code}");

    if cfg!(test) && exit_code > 0x20 {
        let kernel_exit_code = (exit_code >> 1) - 0x10;

        println!("Kernel exited with code {kernel_exit_code}");
    }
}
