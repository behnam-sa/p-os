use bootloader::DiskImageBuilder;
use std::{env, path::PathBuf};

fn main() {
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();
    let disk_builder = DiskImageBuilder::new(kernel_path.into());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("p-os-uefi.img");

    disk_builder.create_uefi_image(&uefi_path).unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
}
