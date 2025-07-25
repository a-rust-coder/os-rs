#![no_std]
#![feature(allocator_api)]

extern crate alloc;

pub mod allocator;

use core::any::Any;

use alloc::{boxed::Box, string::String, vec::Vec};
pub use allocator::*;

pub struct ModulesInfos {
    detected_modules: Vec<String>,
    loaded_modules: Vec<Box<dyn ModuleInterface>>,
}

pub trait ModuleInterface {
    fn request(&self, request: Box<dyn Any>) -> Result<Box<dyn Any>, String>;
    fn save_state(&self) -> Box<dyn Any>;
    fn restore_state(&self, state: Box<dyn Any>) -> Result<(), Box<dyn Any>>;
}
