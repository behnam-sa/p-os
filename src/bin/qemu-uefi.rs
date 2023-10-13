use std::{
    process::{self, Command},
    time::Duration,
};

use wait_timeout::ChildExt;

fn main() {
    run_qemu();
}

#[test]
fn test_kernel() {
    run_qemu();
}

fn run_qemu() {
    let uefi_image = env!("UEFI_IMAGE");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file={uefi_image}"));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    if cfg!(test) {
        qemu.arg("-device")
            .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
        qemu.arg("-serial").arg("stdio");
        qemu.arg("-display").arg("none");
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

    if cfg!(test) && exit_code > 0x20 {
        let kernel_exit_code = (exit_code >> 1) - 0x10;
        process::exit(kernel_exit_code);
    }

    process::exit(exit_code);
}
