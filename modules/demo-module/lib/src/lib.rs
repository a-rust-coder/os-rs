#![no_std]

pub const INTERFACE_NAME: &str = "demo-module";

pub trait DemoModule {
    fn update_number(&self, number: usize) -> usize;
}
