#![no_std]
#![no_main]
#![feature(allocator_api)]

use core::{alloc::Layout, ops::Deref};

use bootloader_api::{entry_point, info::MemoryRegionKind, BootInfo, BootloaderConfig};
use kernel::{log::serial::{serial_init, serial_write_str}, memory::heap::FreeListHeapAllocator, serial_println};

extern crate alloc;
use alloc::{alloc::Allocator, vec::Vec};

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
    serial_println!("Serial... OK");

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

    loop {}
}

entry_point!(start, config = &CONFIG);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("{:#?}", info);
    loop {}
}
