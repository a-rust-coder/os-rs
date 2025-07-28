fn main() {
    let uefi_path = "./target/boot/x86_64/uefi.img";

    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-drive").arg(
            "if=pflash,format=raw,readonly=on,file=target/ovmf/code.fd"
        );
        cmd.arg("-drive").arg(
            "if=pflash,format=raw,file=target/ovmf/vars.fd"
        );
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        cmd.args(["-serial", "stdio"]);
    }
    cmd.arg("-m").arg("4G");
    cmd.arg("-no-reboot");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
