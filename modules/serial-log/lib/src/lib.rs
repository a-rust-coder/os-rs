#![no_std]
#![feature(trait_alias)]

use core::fmt::Write;

pub const INTERFACE_NAME: &str = "serial-log";

pub trait SerialLog = Write;
