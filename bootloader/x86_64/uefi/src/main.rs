#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "win64" fn efi_main(
    _image_handle: usize,
    _system_table: usize,
) -> usize {
    loop {}
}
