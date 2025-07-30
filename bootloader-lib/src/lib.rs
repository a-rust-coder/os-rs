//! TODO: make it FFI-safe (no [u8])

#![no_std]

use core::ptr::NonNull;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootInfo {
    pub kernel_memory: NonNull<[u8]>,
    pub ramdisk_memory: Option<NonNull<[u8]>>,
    pub frame_buffer: Option<FrameBuffer>,
    pub memory_regions: NonNull<[MemoryRegion]>,
    pub physical_memory_offset: Option<usize>,
    pub pixel_format: Option<PixelFormat>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    pub memory: NonNull<[u8]>,
    pub kind: MemoryRegionKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum MemoryRegionKind {
    Kernel,
    Usable,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    U8,
    Unknown {
        red_position: usize,
        green_position: usize,
        blue_position: usize,
    },
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FrameBuffer {
    pub pixel_format: PixelFormat,
    pub buffer: NonNull<[u8]>,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
}
