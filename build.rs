use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let bootloader_x86_64_uefi =
        std::env::var("CARGO_BIN_FILE_BOOTLOADER_X86_64_UEFI_bootloader-x86_64-uefi").unwrap();
    let out_dir = PathBuf::from("target");
    let img_path = out_dir.join("boot/x86_64/uefi.img");


    fs::create_dir_all(out_dir.join("boot/x86_64")).unwrap();
    let _ = fs::remove_file(&img_path);

    let status = Command::new("dd")
        .args(&[
            "if=/dev/zero",
            &format!("of={}", img_path.to_str().unwrap()),
            "bs=1M",
            "count=64",
        ])
        .status()
        .unwrap();
    if !status.success() {
        panic!();
    }

    let status = Command::new("mkfs.vfat")
        .args(&["-F", "32", img_path.to_str().unwrap()])
        .status()
        .unwrap();
    if !status.success() {
        panic!();
    }

    Command::new("mmd")
        .args(&["-i", img_path.to_str().unwrap(), "::/EFI"])
        .status()
        .unwrap();
    Command::new("mmd")
        .args(&["-i", img_path.to_str().unwrap(), "::/EFI/BOOT"])
        .status()
        .unwrap();
    let status = Command::new("mcopy")
        .args(&[
            "-i",
            img_path.to_str().unwrap(),
            &bootloader_x86_64_uefi,
            "::/EFI/BOOT/BOOTX64.EFI",
        ])
        .status()
        .unwrap();
    if !status.success() {
        panic!();
    }
}
