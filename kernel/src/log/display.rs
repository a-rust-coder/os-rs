use core::{fmt::Write, num::NonZero, ptr::NonNull};

use kernel_lib::{boot_info::PixelFormat, BootInfo};

use crate::log::font::FONT8X8_BASIC;

pub struct FrameBufferWriter {
    pub buffer: NonNull<[u8]>,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
    pub x: usize,
    pub y: usize,
    pub fg_color: Color,
    pub bg_color: Color,
    pub color_format: PixelFormat,
}

#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl FrameBufferWriter {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let offset = (y * self.stride + x) * self.bytes_per_pixel;
        if offset + 2 >= self.buffer.len() {
            return;
        }
        let buffer = <NonZero<usize> as Into<usize>>::into(self.buffer.addr()) as *mut u8;

        unsafe {
            match self.color_format {
                PixelFormat::Rgb => {
                    *buffer.add(offset + 0) = color.0;
                    *buffer.add(offset + 1) = color.1;
                    *buffer.add(offset + 2) = color.2;
                }
                PixelFormat::Bgr => {
                    *buffer.add(offset + 0) = color.2;
                    *buffer.add(offset + 1) = color.1;
                    *buffer.add(offset + 2) = color.0;
                }
                PixelFormat::U8 => {
                    *buffer.add(offset) =
                        ((color.0 as u16 * 77 + color.1 as u16 * 150 + color.2 as u16 * 29) >> 8)
                            as u8;
                }
                PixelFormat::Unknown {
                    red_position,
                    green_position,
                    blue_position,
                } => {
                    *buffer.add(offset + red_position) = color.0;
                    *buffer.add(offset + green_position) = color.1;
                    *buffer.add(offset + blue_position) = color.2;
                }
            }
        }
    }

    pub fn draw_char(&mut self, ch: char) {
        let glyph = FONT8X8_BASIC[ch as usize];

        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                let pixel_on = (byte >> col) & 1 != 0;

                let color = if pixel_on {
                    self.fg_color
                } else {
                    self.bg_color
                };
                self.set_pixel(self.x + row, self.y + col, color);
            }
        }

        self.x += 8;
    }

    pub fn newline(&mut self) {
        self.x = 0;
        self.y += 8;
        if self.y + 8 >= self.height {
            self.y = 0
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            if ch == '\n' || self.x + 8 >= self.width {
                self.newline();
            } else {
                self.draw_char(ch);
            }
        }
    }

    pub fn erase(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_pixel(x, y, self.bg_color);
            }
        }
        self.x = 0;
        self.y = 0;
    }
}

impl Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub fn init_framebuffer_writer(boot_info: BootInfo) -> FrameBufferWriter {
    let fb = boot_info.frame_buffer.unwrap();
    FrameBufferWriter {
        buffer: fb.buffer,
        width: fb.width,
        height: fb.height,
        stride: fb.stride,
        bytes_per_pixel: fb.bytes_per_pixel,
        x: 0,
        y: 0,
        fg_color: Color(255, 255, 255),
        bg_color: Color(0, 0, 0),
        color_format: fb.pixel_format,
    }
}
