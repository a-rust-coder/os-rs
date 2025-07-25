#![no_std]
#![no_main]
#![feature(allocator_api, alloc_layout_extra)]

use core::{alloc::Layout, fmt::Write, mem::MaybeUninit};

use alloc::{alloc::Allocator, boxed::Box, vec::Vec};
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use kernel::{
    idt::init_idt,
    log::{Logger, display::init_framebuffer_writer, serial::init_serial},
    memory::{heap::init_heap, init_global_allocator},
    ramdisk::{SimpleInitFs, elf},
    serial_println,
};
use kernel_lib::AllocatorWrapper;
use kernel_proc_macros::log;

extern crate alloc;

const CONFIG: BootloaderConfig = {
    let mut c = BootloaderConfig::new_default();
    c.mappings.page_table_recursive = None;
    c.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    c
};

fn start(boot_info: &mut BootInfo) -> ! {
    init_serial();
    init_idt();
    let allocator = init_heap(boot_info);
    init_global_allocator(allocator);
    let mut writer = init_framebuffer_writer(boot_info);
    writer.erase();
    let mut logger = Logger::new(true, true, Some(writer));
    log!("Hello {}", "world!");
    logger.log_text("Hello world!");
    logger.log_ok("Log");
    logger.log_err("666");

    let (ramdisk_start, ramdisk_len) = (
        boot_info.ramdisk_addr.into_option().unwrap() as usize,
        boot_info.ramdisk_len as usize,
    );
    let fs_bytes = unsafe { core::slice::from_raw_parts(ramdisk_start as *mut u8, ramdisk_len) };

    let fs = SimpleInitFs::new(fs_bytes);

    for file in fs.iter() {
        serial_println!(
            "Found file: {}, size: {}, content: {:?}",
            file.name,
            file.data.len(),
            str::from_utf8(file.data)
        );

        let handle = elf::load_elf(
            file.data,
            allocator
                .allocate(Layout::new::<u8>().repeat(1000000).unwrap().0)
                .unwrap()
                .addr()
                .into(),
        );
        logger.log_ok("Loading ELF");

        let alloc_wrap = AllocatorWrapper(MaybeUninit::new(&allocator));

        let module_glob_alloc = handle.get_symbol("GLOBAL_ALLOCATOR");
        let module_glob_alloc = module_glob_alloc as *mut AllocatorWrapper;
        unsafe { *module_glob_alloc = alloc_wrap };
        log!("Init module GLOBAL_ALLOCATOR");

        let module_fn = handle.get_symbol("module");
        let module_fn: unsafe extern "C" fn(usize, &mut dyn Write) -> Box<usize> =
            unsafe { core::mem::transmute(module_fn) };
        let v = unsafe { module_fn(666, &mut logger) };
        log!("{}", v);
    }

    let mut v = Vec::new();

    for i in 0..10 {
        v.push(i);
    }

    log!("{:?}", v);

    loop {}
}

entry_point!(start, config = &CONFIG);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("{:#?}", info);
    loop {}
}
