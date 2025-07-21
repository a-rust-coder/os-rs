# ```os-rs```

An OS 100% Rust.

First, you should switch to NixOS. Then, run ```nix develop```.

```cargo build``` to generate ```target/bootable/uefi.img``` and ```target/bootable/bios.img```.


## ObjectsManager

The main concept of this OS is the ObjectsManager. In a few words, you can call any function in every object. This allows to change an interface (e.g. the display manager) in the runtime, even if it's a kernel module. Have a look at ```kernel/src/objects.rs```.
