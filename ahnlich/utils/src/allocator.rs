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
        unsafe { self.allocator.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        unsafe { self.allocator.dealloc(ptr, layout) }
    }
}

// When dhat-heap feature is enabled, use dhat's global allocator for memory profiling
#[cfg(feature = "dhat-heap")]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: dhat::Alloc = dhat::Alloc;

// Otherwise, use the custom AhnlichAllocator
#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: AhnlichAllocator<alloc::System> =
    AhnlichAllocator::new(alloc::System, usize::MAX);
