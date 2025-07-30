#![no_std]
#![no_main]

type EfiSatus = u64;
const EFI_SUCCESS: EfiSatus = 0;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    _image_handle: usize,
    _system_table: usize,
) -> EfiSatus {
    EFI_SUCCESS
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

