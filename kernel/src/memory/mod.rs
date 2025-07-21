use core::alloc::GlobalAlloc;

pub mod heap;

pub struct FakeAllocator;

unsafe impl GlobalAlloc for FakeAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: core::alloc::Layout, new_size: usize) -> *mut u8 {
        todo!()
    }
}

#[global_allocator]
pub static FAKE_GLOBAL_ALLOCATOR: FakeAllocator = FakeAllocator;
