#![no_std]

use core::fmt::Write;

pub const INTERFACE_NAME: &str = "serial-log";

pub trait SerialLog {
    fn log_str(&self, s: &str);
}

impl Write for &dyn SerialLog {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.log_str(s);
        Ok(())
    }
}
