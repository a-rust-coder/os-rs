#![no_std]
#![no_main]
#![feature(allocator_api, alloc_layout_extra)]

use core::{
    alloc::Layout,
    fmt::Write,
    mem::{MaybeUninit, transmute},
    panic::PanicInfo,
};

use alloc::alloc::Allocator;
use demo_module_lib::DemoModule;
use kernel::{
    idt::init_idt,
    log::display::init_framebuffer_writer,
    memory::{heap::init_heap, init_global_allocator},
    modules::serial_log,
    ramdisk::{SimpleInitFs, elf},
};
use kernel_lib::{AllocatorWrapper, BootInfo, Module, ModuleHandle, ModuleWrapper};
use kernel_proc_macros::log;
use serial_log_lib::SerialLog;

extern crate alloc;

unsafe extern "C" fn _start(boot_info: BootInfo) -> ! {
    // Init module serial-log (kernel)
    let serial_log: &dyn SerialLog =
        unsafe { transmute(serial_log::MODULE.0.init(&[], boot_info).unwrap().interface) };
    let serial_log_handle = ModuleHandle {
        interface: unsafe { transmute(serial_log) },
        module_name: serial_log::MODULE_NAME,
        interface_name: serial_log::INTERFACE_NAME,
    };
    let mut logger = serial_log;

    log!("\n\n\n\n");
    log!("Module serial-log OK (kernel)");
    log!("Hello world!");

    init_idt();
    let allocator = init_heap(&boot_info);
    init_global_allocator(allocator);

    let mut writer = init_framebuffer_writer(boot_info);
    writer.erase();

    let (ramdisk_start, ramdisk_len) = (
        boot_info.ramdisk_memory.unwrap().as_ptr() as *mut u8 as usize,
        boot_info.ramdisk_memory.unwrap().len(),
    );
    let fs_bytes = unsafe { core::slice::from_raw_parts(ramdisk_start as *mut u8, ramdisk_len) };

    let fs = SimpleInitFs::new(fs_bytes);

    for file in fs.iter() {
        let handle = elf::load_elf(
            file.data,
            allocator
                .allocate(Layout::new::<u8>().repeat(1000000).unwrap().0)
                .unwrap()
                .addr()
                .into(),
        );

        // init GLOBAL_ALLOCATOR
        let allocator_wrap = AllocatorWrapper(MaybeUninit::new(&allocator));
        let glob_alloc_ptr = handle.get_symbol("GLOBAL_ALLOCATOR");
        unsafe { *(glob_alloc_ptr as *mut AllocatorWrapper) = allocator_wrap };

        // init PANIC_HANDLER
        let panic_handler: fn(&PanicInfo) -> ! = panic;
        let panic_handler = MaybeUninit::new(panic_handler);
        let panic_handler_ptr = handle.get_symbol("PANIC_HANDLER");
        unsafe { *(panic_handler_ptr as *mut MaybeUninit<fn(&PanicInfo) -> !>) = panic_handler };

        // get module
        let module_ptr = handle.get_symbol("MODULE");
        let module = unsafe { *(module_ptr as *mut ModuleWrapper) };
        let module = module.0;

        // init module
        let demo_module = module.init(&[serial_log_handle], boot_info).unwrap();
        let demo_module: &dyn DemoModule = unsafe { transmute(demo_module.interface) };
        log!("{}", demo_module.update_number(1));
    }

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
