# ```os-rs```

An OS 100% Rust.

First, you should switch to NixOS. Then, run ```nix develop```.

```cargo build``` to generate ```target/bootable/uefi.img``` and ```target/bootable/bios.img```.

## WARNING

1. Never let an heap-allocated object be dropped in an other module. Even if you know what you're doing. This is bad. Only the kernel is allowed to drop everything.

2. Never consider a module as "safe". This is also bad. Very bad.

3. Don't ```panic!```. This is not a joke, don't ```panic!```, nerver ```.unwrap()```, or similar until the needed module to avoid stay stuck are loaded.

