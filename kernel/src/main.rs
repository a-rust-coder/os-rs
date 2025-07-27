#![no_std]
#![no_main]
#![feature(allocator_api, alloc_layout_extra)]
#![allow(mutable_transmutes)]

use core::{alloc::Layout, fmt::Write, mem::{transmute, MaybeUninit}, panic::PanicInfo};

use alloc::{alloc::Allocator, boxed::Box, vec::Vec};
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use demo_module_lib::DemoModule;
use kernel::{
    idt::init_idt,
    log::{display::init_framebuffer_writer, serial::init_serial, Logger},
    memory::{heap::init_heap, init_global_allocator},
    ramdisk::{elf, SimpleInitFs},
    serial_println,
};
use kernel_lib::{AllocatorWrapper, Module, ModuleWrapper};
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

        // init GLOBAL_ALLOCATOR
        let allocator_wrap = AllocatorWrapper(MaybeUninit::new(&allocator));
        let glob_alloc_ptr = handle.get_symbol("GLOBAL_ALLOCATOR");
        unsafe { *(glob_alloc_ptr as *mut AllocatorWrapper) = allocator_wrap };

        // init PANIC_HANDLER
        let panic_handler: fn(&PanicInfo) -> ! = panic_fn;
        let panic_handler = MaybeUninit::new(panic_handler);
        let panic_handler_ptr = handle.get_symbol("PANIC_HANDLER");
        unsafe { *(panic_handler_ptr as *mut MaybeUninit<fn(&PanicInfo) -> ! >) = panic_handler };

        // get module
        let module_ptr = handle.get_symbol("MODULE");
        let module = unsafe { *(module_ptr as *mut ModuleWrapper) };
        let module = module.0;
        let module: &mut dyn Module = unsafe { transmute(module) };

        // init module
        let demo_module = module.init(&[], boot_info).unwrap();
        let demo_module: &dyn DemoModule = unsafe { transmute(demo_module.interface) };
        serial_println!("{}", demo_module.update_number(1));
    }

    loop {}
}

entry_point!(start, config = &CONFIG);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_fn(info);
}

fn panic_fn(info: &PanicInfo) -> ! {
    serial_println!("{:#?}", info);
    loop {}
}
