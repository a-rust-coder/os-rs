extern crate alloc;

use core::{arch::asm, mem::{transmute, ManuallyDrop}};

use alloc::boxed::Box;
use kernel_lib::{InitOk, Module, ModuleWrapper};
use serial_log_lib::SerialLog;

pub static INTERFACE_NAME: &str = serial_log_lib::INTERFACE_NAME;

pub static MODULE_NAME: &str = "serial-log kernel";

pub static MODULE: ModuleWrapper = ModuleWrapper(&SerialLogMod);

const COM1: u16 = 0x3F8;

pub struct SerialLogMod;

impl Module for SerialLogMod {
    fn init(
        &self,
        _loaded_modules: &[kernel_lib::ModuleHandle],
        _boot_infos: &mut kernel_lib::BootInfo,
    ) -> Result<kernel_lib::InitOk<'_>, kernel_lib::InitErr<'_>> {
        SerialLogMod::outb(COM1 + 1, 0x00); // Disable interrupts
        SerialLogMod::outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        SerialLogMod::outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        SerialLogMod::outb(COM1 + 1, 0x00); //                  (hi byte)
        SerialLogMod::outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        SerialLogMod::outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        SerialLogMod::outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
        //
        let b: Box<dyn SerialLog> = Box::new(SerialLogMod);
        let b = ManuallyDrop::new(b);
        let raw: *const dyn SerialLog = &**b;
        let raw: *mut dyn SerialLog = raw as *mut dyn SerialLog;
        let (data_ptr, vtable_ptr): (*mut (), *mut ()) = unsafe { transmute(raw) };

        Ok(InitOk {
            interface: (data_ptr as usize, vtable_ptr as usize),
            rerun: None,
        })
    }

    fn save_state(&self) -> Box<dyn core::any::Any> {
        Box::new(())
    }

    fn restore_state(
        &self,
        _state: Box<dyn core::any::Any>,
    ) -> Result<(), Box<dyn core::fmt::Debug>> {
        Ok(())
    }

    fn stop(&self) {}
}

impl SerialLogMod {
    pub fn outb(port: u16, val: u8) {
        unsafe {
            asm!("out dx, al", in("dx") port, in ("al") val);
        }
    }

    pub fn inb(port: u16) -> u8 {
        let val: u8;
        unsafe { asm!("in al, dx", out("al") val, in("dx") port) };
        val
    }

    pub fn is_transmit_ready() -> bool {
        Self::inb(COM1 + 5) & 0x20 != 0
    }

    pub fn write_byte(byte: u8) {
        while !Self::is_transmit_ready() {}
        Self::outb(COM1, byte);
    }
}

impl SerialLog for SerialLogMod {
    fn log_str(&self, s: &str) {
        for b in s.as_bytes() {
            Self::write_byte(*b);
        }
    }
}
