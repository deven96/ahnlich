use cap::Cap;
use std::{
    alloc::{self, GlobalAlloc},
    ops::Deref,
};

pub struct AhnlichAllocator<H> {
    allocator: Cap<H>,
}

impl<H> AhnlichAllocator<H> {
    pub const fn new(allocator: H, limit: usize) -> Self {
        Self {
            allocator: Cap::new(allocator, limit),
        }
    }
}

impl<H> Deref for AhnlichAllocator<H> {
    type Target = Cap<H>;
    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

unsafe impl<H> GlobalAlloc for AhnlichAllocator<H>
where
    H: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        //TODO: we reserve 1000 bytes to handle panics
        //const RESERVED: usize = 10000;
        self.allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        self.allocator.dealloc(ptr, layout)
    }
}

#[global_allocator]
pub static GLOBAL_ALLOCATOR: AhnlichAllocator<alloc::System> =
    AhnlichAllocator::new(alloc::System, usize::MAX);
