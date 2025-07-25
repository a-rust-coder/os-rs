use core::{
    alloc::{Allocator, GlobalAlloc}, mem::MaybeUninit, num::NonZero, ptr::NonNull
};

#[repr(transparent)]
pub struct AllocatorWrapper<'a>(pub MaybeUninit<&'a dyn Allocator>);

unsafe impl Allocator for AllocatorWrapper<'_> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        unsafe { self.0.assume_init().allocate(layout) }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.0.assume_init().deallocate(ptr, layout) };
    }
}

unsafe impl GlobalAlloc for AllocatorWrapper<'_> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe {
            <NonZero<usize> as Into<usize>>::into(
                self.0.assume_init().allocate(layout).unwrap().addr(),
            ) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.0.assume_init().deallocate(NonNull::new(ptr).unwrap(), layout) };
    }
}

impl AllocatorWrapper<'_> {
    pub const fn non_init() -> Self {
        Self(MaybeUninit::uninit())
    }
}

unsafe impl Sync for AllocatorWrapper<'_> {}
