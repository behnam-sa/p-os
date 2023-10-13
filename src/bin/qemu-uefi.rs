use std::process::{self, Command};

fn main() {
    let uefi_image = env!("UEFI_IMAGE");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file={uefi_image}"));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
