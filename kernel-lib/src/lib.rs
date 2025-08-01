#![no_std]
#![feature(allocator_api)]

extern crate alloc;

pub mod allocator;
pub mod mutex;
pub mod boot_info;

pub use boot_info::BootInfo;

use core::{any::Any, fmt::Debug};

use alloc::boxed::Box;
pub use allocator::*;

pub trait Module {
    fn init(
        &self,
        loaded_modules: &[ModuleHandle],
        boot_infos: BootInfo,
    ) -> Result<InitOk<'_>, InitErr<'_>>;

    fn save_state(&self) -> Box<dyn Any>;

    fn restore_state(&self, state: Box<dyn Any>) -> Result<(), Box<dyn Debug>>;

    fn stop(&self);
}

#[derive(Clone, Copy)]
pub struct ModuleWrapper<'a>(pub &'a dyn Module);

unsafe impl Sync for ModuleWrapper<'_> {}

#[derive(Debug, Clone, Copy)]
pub struct ModuleHandle<'a> {
    pub interface: (usize, usize),
    pub module_name: &'a str,
    pub interface_name: &'a str,
}

#[derive(Debug)]
pub struct RerunWhen<'a> {
    pub event: Event<'a>,
}

#[derive(Debug)]
pub enum Event<'a> {
    IsLoadedInterface(&'a str),
    IsLoadedModule(&'a str),
    IsLoadedOneOfInterfaces(&'a [&'a str]),
    IsLoadedOneOfModules(&'a [&'a str]),
    And(Box<Event<'a>>, Box<Event<'a>>),
    Or(Box<Event<'a>>, Box<Event<'a>>),
}

#[derive(Debug)]
pub enum InitErr<'a> {
    Rerun(RerunWhen<'a>),
    Error(&'a str),
}

pub struct InitOk<'a> {
    pub interface: (usize, usize),
    pub rerun: Option<RerunWhen<'a>>,
}
