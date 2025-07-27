#![no_std]
#![no_main]

use core::{
    cell::{Cell, UnsafeCell}, fmt::Write, mem::{transmute, ManuallyDrop, MaybeUninit}, panic::PanicInfo
};

use alloc::boxed::Box;
use demo_module_lib::DemoModule;
use kernel_lib::{AllocatorWrapper, InitOk, Module, ModuleWrapper, mutex::Mutex};
use kernel_proc_macros::log;
use serial_log_lib::SerialLog;

extern crate alloc;

#[used]
#[unsafe(no_mangle)]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: AllocatorWrapper = AllocatorWrapper::non_init();

#[used]
#[unsafe(no_mangle)]
pub static PANIC_HANDLER: MaybeUninit<fn(&PanicInfo) -> !> = MaybeUninit::uninit();

#[used]
#[unsafe(no_mangle)]
pub static MODULE_NAME: &str = "demo-module-mod";

#[used]
#[unsafe(no_mangle)]
pub static INTERFACE_NAME: &str = demo_module_lib::INTERFACE_NAME;

#[used]
#[unsafe(no_mangle)]
pub static MODULE: ModuleWrapper = ModuleWrapper(&INITIALIZER);

pub static INITIALIZER: DemoModuleMod = DemoModuleMod {
    n: Mutex::new(1),
    log: MaybeUninit::uninit(),
};

pub struct DemoModuleMod<'a> {
    n: Mutex<usize>,
    log: MaybeUninit<&'a dyn SerialLog>,
}

unsafe impl Sync for DemoModuleMod<'_> {}

impl Module for DemoModuleMod<'_> {
    fn init(
        &self,
        loaded_modules: &[kernel_lib::ModuleHandle],
        _boot_infos: &mut kernel_lib::BootInfo,
    ) -> Result<kernel_lib::InitOk<'_>, kernel_lib::InitErr<'_>> {
        let mut log: MaybeUninit<&dyn SerialLog> = MaybeUninit::uninit();

        for m in loaded_modules {
            if m.interface_name == "serial-log" {
                log = MaybeUninit::new(unsafe { transmute(m.interface) });
            }
        }

        let b: Box<dyn DemoModule> = Box::new(DemoModuleMod {
            n: Mutex::new(1),
            log,
        });
        let b = ManuallyDrop::new(b);
        let raw: *const dyn DemoModule = &**b;
        let raw: *mut dyn DemoModule = raw as *mut dyn DemoModule;
        let (data_ptr, vtable_ptr): (*mut (), *mut ()) = unsafe { transmute(raw) };

        Ok(InitOk {
            interface: (data_ptr as usize, vtable_ptr as usize),
            rerun: None,
        })
    }

    fn save_state(&self) -> alloc::boxed::Box<dyn core::any::Any> {
        Box::new(*self.n.lock())
    }

    fn restore_state(
        &self,
        state: Box<dyn core::any::Any>,
    ) -> Result<(), Box<dyn core::fmt::Debug>> {
        match state.downcast::<usize>() {
            Ok(v) => {
                *self.n.lock() = *v;
                return Ok(());
            }
            Err(_) => {
                return Err(Box::new("Type error"));
            }
        }
    }

    fn stop(&self) {}
}

impl DemoModule for DemoModuleMod<'_> {
    fn update_number(&self, number: usize) -> usize {
        let mut logger = unsafe { self.log.assume_init() };
        log!("Number: {}", number);
        *self.n.lock() + number
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { PANIC_HANDLER.assume_init()(info) };
}
