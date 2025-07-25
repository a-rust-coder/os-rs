use core::{alloc::{Allocator, GlobalAlloc}, mem::MaybeUninit, num::NonZero, ptr::NonNull};

use crate::memory::heap::FreeListHeapAllocator;

pub mod heap;
pub mod page_table;

#[repr(transparent)]
pub struct AllocatorWrapper(FreeListHeapAllocator);

unsafe impl Allocator for AllocatorWrapper {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        self.0.allocate(layout) 
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.0.deallocate(ptr, layout) };
    }
}

unsafe impl GlobalAlloc for AllocatorWrapper {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe {
            <NonZero<usize> as Into<usize>>::into(
                self.0.allocate(layout).unwrap().addr(),
            ) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.0.deallocate(NonNull::new(ptr).unwrap(), layout) };
    }
}

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: AllocatorWrapper = AllocatorWrapper(FreeListHeapAllocator(unsafe { NonNull::new(core::slice::from_raw_parts_mut(1 as *mut u8, 0)).unwrap() }));

pub fn init_global_allocator(ga: FreeListHeapAllocator) {
    unsafe { GLOBAL_ALLOCATOR.0 = ga };
}
