use core::{
    alloc::{AllocError, Allocator},
    num::NonZero,
    ptr::{self, NonNull},
    usize,
};

use crate as kernel;
use crate::serial_println;

#[derive(Debug, Clone, Copy)]
pub struct FreeListHeapAllocator(NonNull<[u8]>);

impl FreeListHeapAllocator {
    pub fn new(start: usize, size: usize) -> Self {
        let second = UnusedRegion::new(start + 24, size - 24, None);
        UnusedRegion::new(start, 0, Some(second));
        Self(NonNull::new(ptr::slice_from_raw_parts_mut(start as *mut u8, size)).unwrap())
    }

    pub fn show_state(&self) {
        serial_println!("Showing allocator (FreeListHeapAllocator) memory state:");
        serial_println!("Size: {}", self.0.len());
        serial_println!("Address: {}", self.0.addr());

        let infos = UnusedRegion::read(self.0.addr().into());
        let mut current_unused_region = match infos.next() {
            None => {
                serial_println!("-- Full --");
                return;
            }
            Some(v) => v,
        };

        loop {
            serial_println!("------------ UNUSED REGION -------------");
            serial_println!(
                "Position: {}",
                current_unused_region.address
                    - <NonZero<usize> as Into<usize>>::into(self.0.addr())
            );
            serial_println!("Size: {}", current_unused_region.size());
            serial_println!(
                "Next position: {}",
                (current_unused_region.infos.next as isize)
                    - (<NonZero<usize> as Into<usize>>::into(self.0.addr()) as isize)
            );
            serial_println!("Next address: {}", current_unused_region.infos.next);
            serial_println!();

            match current_unused_region.next() {
                None => {
                    serial_println!("END OF UNUSED REGIONS");
                    return;
                }
                Some(v) => current_unused_region = v,
            }
        }
    }
}

unsafe impl Allocator for FreeListHeapAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let infos = UnusedRegion::read(self.0.addr().into());
        let mut current_unused_region = match infos.next() {
            None => return Err(AllocError),
            Some(v) => v,
        };
        let (mut l_size, l_align) = (layout.size(), layout.align());

        if l_size < 16 {
            l_size = 16
        }

        loop {
            let pad = (l_align - (current_unused_region.address % l_align)) % l_align;

            if pad == 0 && l_size == current_unused_region.size() {
                let mut prev = current_unused_region.get_prev().unwrap();
                prev.set_next(current_unused_region.next());
                return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
                    current_unused_region.address as *mut u8,
                    l_size,
                ))
                .unwrap());
            }

            if pad == 0 && l_size + 16 <= current_unused_region.size() {
                let mut prev = current_unused_region.get_prev().unwrap();
                let new = UnusedRegion::new(
                    current_unused_region.address + l_size,
                    current_unused_region.size() - l_size,
                    current_unused_region.next(),
                );
                prev.set_next(Some(new));
                return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
                    current_unused_region.address as *mut u8,
                    l_size,
                ))
                .unwrap());
            }

            if pad >= 16 && pad + l_size == current_unused_region.size() {
                current_unused_region.set_size(pad);
                return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
                    (current_unused_region.address + pad) as *mut u8,
                    l_size,
                ))
                .unwrap());
            }

            if pad >= 16 && pad + l_size + 16 <= current_unused_region.size() {
                let new = UnusedRegion::new(
                    current_unused_region.address + pad + l_size,
                    current_unused_region.size() - pad - l_size,
                    current_unused_region.next(),
                );
                current_unused_region.set_size(pad);
                current_unused_region.set_next(Some(new));
                return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
                    (current_unused_region.address + pad) as *mut u8,
                    l_size,
                ))
                .unwrap());
            }

            let pad = {
                let mut pad = pad;
                while pad < 16 {
                    pad += l_align;
                }
                pad
            };

            if pad + l_size + 16 <= current_unused_region.size() {
                let new = UnusedRegion::new(
                    current_unused_region.address + pad + l_size,
                    current_unused_region.size() - pad - l_size,
                    current_unused_region.next(),
                );
                current_unused_region.set_size(pad);
                current_unused_region.set_next(Some(new));
                return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
                    (current_unused_region.address + pad) as *mut u8,
                    l_size,
                ))
                .unwrap());
            }

            match current_unused_region.next() {
                None => return Err(AllocError),
                Some(r) => current_unused_region = r,
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        let (address, mut size): (usize, usize) = (ptr.addr().into(), layout.size());

        if size < 16 {
            size = 16
        }

        let mut infos = UnusedRegion::read(self.0.addr().into());
        let mut current_unused_region = match infos.next() {
            None => {
                let new = UnusedRegion::new(address, size, None);
                infos.set_next(Some(new));
                return;
            }
            Some(v) => v,
        };

        loop {
            if current_unused_region.address > address {
                let mut prev = current_unused_region.get_prev().unwrap();

                let adjacent_left = prev.address + prev.size() == address;
                let adjacent_right = address + size == current_unused_region.address;

                if adjacent_left && adjacent_right {
                    prev.set_size(prev.size() + size + current_unused_region.size());
                    prev.set_next(current_unused_region.next());
                } else if adjacent_left {
                    prev.set_size(prev.size() + size);
                } else if adjacent_right {
                    let new = UnusedRegion::new(
                        address,
                        size + current_unused_region.size(),
                        current_unused_region.next(),
                    );
                    prev.set_next(Some(new));
                } else {
                    let new = UnusedRegion::new(address, size, Some(current_unused_region));
                    prev.set_next(Some(new));
                }
                return;
            }

            match current_unused_region.next() {
                None => {
                    let mut prev = current_unused_region.get_prev().unwrap();

                    if prev.address + prev.size() == address {
                        prev.set_size(prev.size() + size);
                    } else {
                        let new = UnusedRegion::new(address, size, None);
                        prev.set_next(Some(new));
                    }
                    return;
                }
                Some(r) => {
                    current_unused_region = r;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct UnusedRegionInfos {
    pub size: usize,
    pub next: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnusedRegion {
    infos: UnusedRegionInfos,
    address: usize,
    prev: Option<usize>,
}

impl UnusedRegion {
    pub const fn read(address: usize) -> Self {
        Self {
            infos: unsafe { ptr::read_unaligned(address as *mut UnusedRegionInfos) },
            address,
            prev: None,
        }
    }

    pub const fn new(address: usize, size: usize, next: Option<UnusedRegion>) -> Self {
        let infos = UnusedRegionInfos {
            size: size,
            next: match next {
                Some(v) => v.address,
                None => 0,
            },
        };
        unsafe { ptr::write_unaligned(address as *mut UnusedRegionInfos, infos) };
        Self {
            infos,
            address,
            prev: None,
        }
    }

    pub const fn size(&self) -> usize {
        self.infos.size
    }

    pub const fn with_prev(mut self, prev: Self) -> Self {
        self.prev = Some(prev.address);
        self
    }

    pub const fn next(&self) -> Option<Self> {
        if self.infos.next == 0 {
            None
        } else {
            Some(Self::read(self.infos.next).with_prev(*self))
        }
    }

    pub const fn rewrite(&self) {
        unsafe { ptr::write_unaligned(self.address as *mut UnusedRegionInfos, self.infos) };
    }

    pub const fn set_size(&mut self, new_size: usize) {
        self.infos.size = new_size;
        self.rewrite();
    }

    pub const fn set_next(&mut self, new_next: Option<Self>) {
        self.infos.next = match new_next {
            Some(n) => n.address,
            None => 0,
        };
        self.rewrite();
    }

    pub const fn get_prev(&self) -> Option<Self> {
        match self.prev {
            Some(v) => Some(Self::read(v)),
            None => None,
        }
    }
}
