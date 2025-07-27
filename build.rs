use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from("./target/bootable/");
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    let test_modules = std::env::var_os("CARGO_BIN_FILE_DEMO_MODULE_MOD_demo-module-mod").unwrap();

    let out = File::create("ramdisk.img").unwrap();
    let mut writer = BufWriter::new(out);

    let paths: Vec<(OsString, &str)> = vec![
        (test_modules, "test.elf"),
    ];
    for path in paths {
        let data = fs::read(path.0).unwrap();
        write_entry(&mut writer, path.1, &data).unwrap();
    }

    writer.flush().unwrap();
    drop(writer);

    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_ramdisk(Path::new("./ramdisk.img"))
        .create_disk_image(&uefi_path)
        .unwrap();

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .set_ramdisk(Path::new("./ramdisk.img"))
        .create_disk_image(&bios_path)
        .unwrap();
}

fn write_entry<W: Write>(writer: &mut W, name: &str, data: &[u8]) -> std::io::Result<()> {
    writer.write_all(&(name.len() as u64).to_le_bytes())?;
    writer.write_all(name.as_bytes())?;
    writer.write_all(&(data.len() as u64).to_le_bytes())?;
    writer.write_all(data)?;
    Ok(())
}
