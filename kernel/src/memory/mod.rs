use core::alloc::GlobalAlloc;

pub mod heap;

pub struct FakeAllocator;

unsafe impl GlobalAlloc for FakeAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout, _new_size: usize) -> *mut u8 {
        todo!()
    }
}

#[global_allocator]
pub static FAKE_GLOBAL_ALLOCATOR: FakeAllocator = FakeAllocator;
