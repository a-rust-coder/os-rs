use std::process::Command;

fn main() {
    let img_path = "./target/boot/x86_64/uefi.img";

    let uefi = true;

    let mut cmd = Command::new("qemu-system-x86_64");

    if uefi {
        cmd.args(&[
            "-drive",
            "if=pflash,format=raw,readonly=on,file=target/ovmf/code.fd",
            "-drive",
            "if=pflash,format=raw,file=target/ovmf/vars.fd",
            "-drive",
            &format!("format=raw,file={}", img_path),
            "-serial",
            "stdio",
        ]);
    }

    cmd.args(&["-m", "4G", "-no-reboot"]);

    let mut child = cmd.spawn().expect("Failed to spawn QEMU");
    child.wait().expect("Failed to wait on QEMU");
}
