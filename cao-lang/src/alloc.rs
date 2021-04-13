use std::{
    alloc::{alloc, dealloc, Layout},
    cell::UnsafeCell,
    ptr::NonNull,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum AllocError {
    #[error("The allocator is out of memory")]
    OutOfMemory,
}

pub struct BumpAllocator {
    data: NonNull<u8>,
    capacity: usize,
    head: UnsafeCell<usize>,
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.data.as_ptr(),
                Layout::from_size_align(self.capacity, 8).expect("Failed to produce alignment"),
            );
        }
    }
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        unsafe {
            Self {
                data: NonNull::new(alloc(
                    Layout::from_size_align(capacity, 8).expect("Failed to produce alignment"),
                ))
                .expect("Failed to allocate memory"),
                capacity,
                head: UnsafeCell::new(0),
            }
        }
    }

    ///# SAFETY
    ///
    ///Invalidates all outstanding pointers
    pub unsafe fn reset(&mut self, capacity: usize) {
        unsafe {
            dealloc(
                self.data.as_ptr(),
                Layout::from_size_align(self.capacity, 8).expect("Failed to produce alignment"),
            );

            self.data = NonNull::new(alloc(
                Layout::from_size_align(capacity, 8).expect("Failed to produce alignment"),
            ))
            .expect("Failed to allocate memory");
            self.capacity = capacity;
            *self.head.get_mut() = 0;
        }
    }

    ///# SAFETY
    ///
    ///Invalidates all outstanding pointers
    pub unsafe fn clear(&mut self) {
        *self.head.get_mut() = 0;
    }

    /// # SAFETY
    /// `alloc` is not thread safe. It is on the caller to ensure that only a single thread uses
    /// the allocator at a time
    pub unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        let s = l.size() + l.align();
        if *self.head.get() + s >= self.capacity {
            return Err(AllocError::OutOfMemory);
        }
        let ptr = self.data.as_ptr().add(*self.head.get());
        *self.head.get() += s;
        let ptr = ptr.add(ptr.align_offset(l.align()));
        Ok(NonNull::new_unchecked(ptr))
    }

    /// # SAFETY
    ///
    /// Only pointers allocated by this instance are safe to free
    pub unsafe fn dealloc(&self, _p: NonNull<u8>) {
        // noop
    }
}
