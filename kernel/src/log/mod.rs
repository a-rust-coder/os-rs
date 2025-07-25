use core::fmt::Write;

use crate::{log::display::{Color, FrameBufferWriter}, serial_print, serial_println};

pub mod display;
pub mod font;
pub mod serial;

pub struct Logger {
    log_to_serial: bool,
    log_to_screen: bool,
    frame_buffer_writer: Option<FrameBufferWriter>,
}

impl Logger {
    pub fn new(
        log_to_serial: bool,
        log_to_screen: bool,
        frame_buffer_writer: Option<FrameBufferWriter>,
    ) -> Self {
        Self {
            log_to_serial,
            log_to_screen,
            frame_buffer_writer,
        }
    }

    pub fn log_text(&mut self, text: &str) {
        use crate as kernel;
        if self.log_to_serial {
            serial_println!("LOG: {}", text);
        }
        if self.log_to_screen {
            if let Some(fbw) = &mut self.frame_buffer_writer {
                fbw.fg_color = Color(255, 255, 255);
                fbw.bg_color = Color(0, 0, 0);
                fbw.write_fmt(format_args!("LOG: {}\n", text)).unwrap();
            }
        }
    }

    pub fn log_ok(&mut self, text: &str) {
        use crate as kernel;
        if self.log_to_serial {
            serial_println!("{} ... [ \x1b[32mOK\x1b[0m ]", text);
        }
        if self.log_to_screen {
            if let Some(fbw) = &mut self.frame_buffer_writer {
                fbw.write_str(text);
                fbw.write_str(" ... [ ");
                fbw.fg_color = Color(0, 255, 0);
                fbw.bg_color = Color(0, 0, 0);
                fbw.write_str("OK");
                fbw.fg_color = Color(255, 255, 255);
                fbw.write_str(" ]\n");
            }
        }
    }

    pub fn log_err(&mut self, text: &str) {
        use crate as kernel;
        if self.log_to_serial {
            serial_println!("{} ... [\x1b[31mERR\x1b[0m ]", text);
        }
        if self.log_to_screen {
            if let Some(fbw) = &mut self.frame_buffer_writer {
                fbw.write_str(text);
                fbw.write_str(" ... [");
                fbw.fg_color = Color(255, 0, 0);
                fbw.bg_color = Color(0, 0, 0);
                fbw.write_str("ERR");
                fbw.fg_color = Color(255, 255, 255);
                fbw.write_str(" ]\n");
            }
        }
    }
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        use crate as kernel;
        if self.log_to_serial {
            serial_print!("{}", s);
        }
        if self.log_to_screen {
            if let Some(fbw) = &mut self.frame_buffer_writer {
                fbw.write_str(s);
            }
        }
        Ok(())
    }
}
