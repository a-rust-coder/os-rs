#![no_std]
#![no_main]

// use core::ffi::c_void;
// use core::panic::PanicInfo;
// use core::ptr::null_mut;
//
// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct Guid {
//     data1: u32,
//     data2: u16,
//     data3: u16,
//     data4: [u8; 8],
// }
//
// type EfiHandle = *mut c_void;
// type EfiStatus = usize;
//
// const EFI_SUCCESS: EfiStatus = 0;
//
// #[repr(C)]
// pub struct EfiTableHeader {
//     signature: u64,
//     revision: u32,
//     header_size: u32,
//     crc32: u32,
//     reserved: u32,
// }
//
// #[repr(C)]
// pub struct EfiSystemTable {
//     hdr: EfiTableHeader,
//     firmware_vendor: *mut u16,
//     firmware_revision: u32,
//     console_in_handle: EfiHandle,
//     con_in: *mut c_void,
//     console_out_handle: EfiHandle,
//     con_out: *mut c_void,
//     standard_error_handle: EfiHandle,
//     std_err: *mut c_void,
//     runtime_services: *mut c_void,
//     boot_services: *mut EfiBootServices,
//     number_of_table_entries: usize,
//     configuration_table: *mut c_void,
// }
//
// #[repr(C)]
// pub struct EfiBootServices {
//     _pad1: [usize; 19],
//     locate_protocol: unsafe extern "efiapi" fn(
//         protocol: *const Guid,
//         registration: *mut c_void,
//         interface: *mut *mut c_void,
//     ) -> EfiStatus,
//     _pad2: [usize; 3],
//     allocate_pages: unsafe extern "efiapi" fn(
//         alloc_type: u32,
//         mem_type: u32,
//         pages: usize,
//         memory: *mut usize,
//     ) -> EfiStatus,
// }
//
// #[repr(C)]
// pub struct EfiSimpleFileSystemProtocol {
//     revision: u64,
//     open_volume: unsafe extern "efiapi" fn(
//         this: *mut EfiSimpleFileSystemProtocol,
//         root: *mut *mut EfiFileProtocol,
//     ) -> EfiStatus,
// }
//
// #[repr(C)]
// pub struct EfiFileProtocol {
//     revision: u64,
//     open: unsafe extern "efiapi" fn(
//         this: *mut EfiFileProtocol,
//         new_handle: *mut *mut EfiFileProtocol,
//         file_name: *const u16,
//         open_mode: u64,
//         attributes: u64,
//     ) -> EfiStatus,
//     close: unsafe extern "efiapi" fn(this: *mut EfiFileProtocol) -> EfiStatus,
//     delete: unsafe extern "efiapi" fn(this: *mut EfiFileProtocol) -> EfiStatus,
//     read: unsafe extern "efiapi" fn(
//         this: *mut EfiFileProtocol,
//         buffer_size: *mut usize,
//         buffer: *mut c_void,
//     ) -> EfiStatus,
//     write: unsafe extern "efiapi" fn(
//         this: *mut EfiFileProtocol,
//         buffer_size: *mut usize,
//         buffer: *const c_void,
//     ) -> EfiStatus,
//     get_position: *mut c_void,
//     set_position: *mut c_void,
//     get_info: unsafe extern "efiapi" fn(
//         this: *mut EfiFileProtocol,
//         info_type: *const Guid,
//         buffer_size: *mut usize,
//         buffer: *mut c_void,
//     ) -> EfiStatus,
//     set_info: *mut c_void,
//     flush: *mut c_void,
// }
//
// #[repr(C)]
// pub struct EfiFileInfo {
//     size: u64,
//     file_size: u64,
//     physical_size: u64,
//     create_time: [u16; 6],
//     last_access_time: [u16; 6],
//     modification_time: [u16; 6],
//     attribute: u64,
//     file_name: [u16; 256],
// }
//
// // Protocol GUIDs
// const SIMPLE_FS_PROTOCOL_GUID: Guid = Guid {
//     data1: 0x0964e5b22,
//     data2: 0x6459,
//     data3: 0x11d2,
//     data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
// };
//
// const FILE_INFO_GUID: Guid = Guid {
//     data1: 0x09576e92,
//     data2: 0x6d3f,
//     data3: 0x11d2,
//     data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
// };
//
// // File mode flags
// const EFI_FILE_MODE_READ: u64 = 0x0000000000000001;
//
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    _image_handle: usize,//EfiHandle,
    system_table: usize//*mut EfiSystemTable,
) -> usize {//EfiStatus {
    loop {}
    // unsafe {
    //     let st = &*system_table;
    //     let bs = &*st.boot_services;
    //
    //     // Locate SimpleFileSystem
    //     let mut sfs_ptr: *mut c_void = null_mut();
    //     let status = (bs.locate_protocol)(&SIMPLE_FS_PROTOCOL_GUID, null_mut(), &mut sfs_ptr);
    //
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     let sfs = &mut *(sfs_ptr as *mut EfiSimpleFileSystemProtocol);
    //
    //     // Open volume (root directory)
    //     let mut root: *mut EfiFileProtocol = null_mut();
    //     let status = (sfs.open_volume)(sfs, &mut root);
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     // UTF-16 "kernel.elf"
    //     let file_name_utf16: [u16; 11] = [
    //         'k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '.' as u16,
    //         'e' as u16, 'l' as u16, 'f' as u16, 0,
    //     ];
    //
    //     let mut file: *mut EfiFileProtocol = null_mut();
    //     let status = ((*root).open)(
    //         root,
    //         &mut file,
    //         file_name_utf16.as_ptr(),
    //         EFI_FILE_MODE_READ,
    //         0,
    //     );
    //
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     // Get file size via get_info
    //     let mut file_info_buf = [0u8; 512];
    //     let mut buf_size = file_info_buf.len();
    //
    //     let status = ((*file).get_info)(
    //         file,
    //         &FILE_INFO_GUID,
    //         &mut buf_size,
    //         file_info_buf.as_mut_ptr() as *mut c_void,
    //     );
    //
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     let file_info = &*(file_info_buf.as_ptr() as *const EfiFileInfo);
    //     let file_size = file_info.file_size as usize;
    //     let pages = (file_size + 0xFFF) / 0x1000;
    //
    //     let mut kernel_addr: usize = 0;
    //     let status = (bs.allocate_pages)(1, 4, pages, &mut kernel_addr);
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     let mut read_size = file_size;
    //     let status = ((*file).read)(file, &mut read_size, kernel_addr as *mut c_void);
    //
    //     if status != EFI_SUCCESS {
    //         return status;
    //     }
    //
    //     // SUCCESS — le fichier est en RAM à kernel_addr
    //     loop {} // bloc infini pour ne pas quitter
    //
    //     // EFI_SUCCESS
    // }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { *(0 as *mut u8) };
    loop {}
}
