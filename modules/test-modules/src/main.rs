#![no_std]
#![no_main]
#![allow(improper_ctypes_definitions)]

extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use alloc::boxed::Box;
use kernel_lib::AllocatorWrapper;
use kernel_proc_macros::log;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
#[unsafe(no_mangle)]
#[used]
pub static GLOBAL_ALLOCATOR: AllocatorWrapper = AllocatorWrapper::non_init();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn module(v: usize, logger: &mut dyn Write) -> Box<usize> {
    log!("Value: {}", v);
    Box::new(v)
}

#[unsafe(no_mangle)]
#[used]
pub static _KEEP: unsafe extern "C" fn(usize, &mut dyn Write) -> Box<usize> = module;
