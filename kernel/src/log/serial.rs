use core::arch::asm;

pub const COM1: u16 = 0x3F8;

pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in ("al") val);
    }
}

pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { asm!("in al, dx", out("al") val, in("dx") port) };
    val
}

pub fn serial_init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(COM1 + 1, 0x00); //                  (hi byte)
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

pub unsafe fn serial_is_transmit_ready() -> bool {
    unsafe { inb(COM1 + 5) & 0x20 != 0 }
}

unsafe fn serial_write_byte(byte: u8) {
    unsafe {
        while !serial_is_transmit_ready() {}
        outb(COM1, byte);
    }
}

pub unsafe fn serial_write_str(s: &str) {
    for b in s.bytes() {
        unsafe { serial_write_byte(b) };
    }
}

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        SerialPort { port }
    }

    pub unsafe fn init(&self) {
        unsafe {
            outb(self.port + 1, 0x00);
            outb(self.port + 3, 0x80);
            outb(self.port + 0, 0x03);
            outb(self.port + 1, 0x00);
            outb(self.port + 3, 0x03);
            outb(self.port + 2, 0xC7);
            outb(self.port + 4, 0x0B);
        }
    }

    fn is_transmit_ready(&self) -> bool {
        unsafe { inb(self.port + 5) & 0x20 != 0 }
    }

    fn write_byte(&self, byte: u8) {
        while !self.is_transmit_ready() {}
        unsafe { outb(self.port, byte) }
    }
}

use core::fmt::Write;

impl Write for &SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

use core::fmt::Arguments;

static SERIAL1: SerialPort = SerialPort::new(0x3F8);

#[doc(hidden)]
pub fn _print(args: Arguments) {
    let mut writer = &SERIAL1;
    let _ = writer.write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        kernel::log::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n");
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\n"), $($arg)*);
    };
}

