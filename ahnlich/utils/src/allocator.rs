use cap::Cap;
use std::alloc;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);
