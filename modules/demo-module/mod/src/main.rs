#![no_std]
#![no_main]

use core::{mem::{transmute, ManuallyDrop, MaybeUninit}, panic::PanicInfo};

use alloc::boxed::Box;
use demo_module_lib::DemoModule;
use kernel_lib::{AllocatorWrapper, InitOk, Module, ModuleWrapper};

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
pub static MODULE: ModuleWrapper = ModuleWrapper(&DemoModuleMod(1));

struct DemoModuleMod(usize);

impl Module for DemoModuleMod {
    fn init(
        &mut self,
        _loaded_modules: &[kernel_lib::ModuleHandle],
        _boot_infos: &mut kernel_lib::BootInfo,
    ) -> Result<kernel_lib::InitOk<'_>, kernel_lib::InitErr<'_>> {
        let b: Box<dyn DemoModule> = Box::new(DemoModuleMod(1));
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
        Box::new(self.0)
    }

    fn restore_state(
        &mut self,
        state: Box<dyn core::any::Any>,
    ) -> Result<(), Box<dyn core::fmt::Debug>> {
        match state.downcast::<usize>() {
            Ok(v) => {
                self.0 = *v;
                return Ok(());
            }
            Err(_) => {
                return Err(Box::new("Type error"));
            }
        }
    }
}

impl DemoModule for DemoModuleMod {
    fn update_number(&self, number: usize) -> usize {
        self.0 + number
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { PANIC_HANDLER.assume_init()(info) };
}
