use ovmf_prebuilt::{Arch, FileType};

fn main() {
    let prebuilt =
        ovmf_prebuilt::Prebuilt::fetch(ovmf_prebuilt::Source::LATEST, "target/ovmf").unwrap();
    let uefi_path = "./target/bootable/uefi.img";
    let bios_path = "./target/bootable/bios.img";

    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-drive").arg(
            "if=pflash,format=raw,readonly=on,file=".to_owned()
                + prebuilt
                    .get_file(Arch::X64, FileType::Code)
                    .to_str()
                    .unwrap(),
        );
        cmd.arg("-drive").arg(
            "if=pflash,format=raw,file=".to_owned()
                + prebuilt
                    .get_file(Arch::X64, FileType::Vars)
                    .to_str()
                    .unwrap(),
        );
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        cmd.args(["-serial", "stdio"]);
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }
    cmd.arg("-m").arg("4G");
    cmd.arg("-no-reboot");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
