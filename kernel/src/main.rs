#![no_std]
#![no_main]
#![feature(allocator_api)]

use core::{fmt::Write, ops::Deref, ptr::NonNull};

use bootloader_api::{BootInfo, BootloaderConfig, entry_point, info::MemoryRegionKind};
use kernel::{
    log::{
        display::{Color, FrameBufferWriter},
        serial::serial_init, Logger,
    }, memory::heap::FreeListHeapAllocator, ramdisk::SimpleInitFs, serial_println
};

extern crate alloc;

const CONFIG: BootloaderConfig = {
    let mut c = BootloaderConfig::new_default();
    c.mappings.page_table_recursive = None;
    c.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    c
};

fn start(boot_info: &mut BootInfo) -> ! {
    unsafe {
        serial_init();
    }
    serial_println!();
    serial_println!();
    serial_println!("Serial... ok");

    let mut biggest_address = 0;
    let mut biggest_size = 0;

    let mut current_address = 0;
    let mut current_size = 0;

    for region in boot_info.memory_regions.deref() {
        if region.kind == MemoryRegionKind::Usable {
            if current_size > 0 {
                current_size += region.end - region.start;
            } else {
                current_size = region.end - region.start;
                current_address = region.start;
            }
        } else {
            if current_size > biggest_size {
                biggest_size = current_size;
                biggest_address = current_address;
            }
            current_size = 0;
            current_address = 0;
        }
    }

    let phys_memory_offset = boot_info.physical_memory_offset.into_option().unwrap() as usize;

    let heap_start = biggest_address as usize + phys_memory_offset;
    let heap_size = 1048576;

    let allocator = FreeListHeapAllocator::new(heap_start, heap_size);

    serial_println!("Allocator... ok");

    let fb = boot_info.framebuffer.as_mut().unwrap();
    let info = fb.info();

    let mut writer = FrameBufferWriter {
        buffer: NonNull::<[u8]>::new(fb.buffer_mut()).unwrap(),
        width: info.width,
        height: info.height,
        stride: info.stride,
        bytes_per_pixel: info.bytes_per_pixel,
        x: 0,
        y: 0,
        fg_color: Color(255, 255, 255),
        bg_color: Color(0, 0, 0),
        color_format: info.pixel_format,
    };

    writer.write_str("Framebuffer... ok\n");
    serial_println!("Framebuffer... ok");

    writer.erase();

    let mut logger = Logger::new(true, true, Some(writer));
    logger.log_text("Hello world!");
    logger.log_ok("Log");
    logger.log_err("666");

    serial_println!("{:#?}", boot_info);

    let (ramdisk_start, ramdisk_len) = (boot_info.ramdisk_addr.into_option().unwrap() as usize, boot_info.ramdisk_len as usize);
    let fs_bytes = unsafe { core::slice::from_raw_parts(ramdisk_start as *mut u8, ramdisk_len) };

    let fs = SimpleInitFs::new(fs_bytes);

    for file in fs.iter() {
        serial_println!("Found file: {}, size: {}, content: {:?}", file.name, file.data.len(), str::from_utf8(file.data));
    }

    loop {}
}

entry_point!(start, config = &CONFIG);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("{:#?}", info);
    loop {}
}
