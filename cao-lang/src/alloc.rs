mod bump_alloc;
pub use bump_alloc::*;

use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::NonNull,
};

// TODO: replace w/ standard traits once they are stabilized
pub trait Allocator {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError>;
    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout);
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AllocError {
    #[error("The allocator is out of memory")]
    OutOfMemory,
}

/// Calls system functions
#[derive(Debug, Default, Clone, Copy)]
pub struct SysAllocator;

impl Allocator for SysAllocator {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        let res = alloc(l);
        if res.is_null() {
            return Err(AllocError::OutOfMemory);
        }
        Ok(NonNull::new_unchecked(res))
    }

    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        dealloc(p.as_ptr(), l);
    }
}

unsafe impl Send for SysAllocator {}
unsafe impl Sync for SysAllocator {}
