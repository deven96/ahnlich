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

#[cfg(not(feature = "dhat-heap"))]
pub fn check_memory_available(estimated_bytes: usize) -> Result<(), AllocationError> {
    let current = GLOBAL_ALLOCATOR.allocated();
    let limit = GLOBAL_ALLOCATOR.limit();

    // Add 10% safety margin to account for allocation overhead
    let needed = estimated_bytes + (estimated_bytes / 10);

    if current + needed > limit {
        return Err(AllocationError::ExceedsLimit {
            requested: estimated_bytes,
            available: limit.saturating_sub(current),
        });
    }

    Ok(())
}

#[cfg(feature = "dhat-heap")]
pub fn check_memory_available(_estimated_bytes: usize) -> Result<(), AllocationError> {
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    ExceedsLimit { requested: usize, available: usize },
}

impl std::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocationError::ExceedsLimit {
                requested,
                available,
            } => {
                write!(
                    f,
                    "allocation of {} bytes would exceed limit (only {} bytes available)",
                    requested, available
                )
            }
        }
    }
}

impl std::error::Error for AllocationError {}

impl From<AllocationError> for std::collections::TryReserveError {
    fn from(_: AllocationError) -> Self {
        Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err()
    }
}

impl From<AllocationError> for fallible_collections::TryReserveError {
    fn from(_: AllocationError) -> Self {
        fallible_collections::TryReserveError::CapacityOverflow
    }
}
