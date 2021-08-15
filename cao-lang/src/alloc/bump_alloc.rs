use super::{AllocError, Allocator};
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::UnsafeCell,
    ops::Deref,
    ptr::NonNull,
    rc::Rc,
};

/// Shared BumpAllocator.
///
/// # Safety
///
/// Note that BumpAllocator is NOT thread-safe!
#[derive(Debug, Clone)]
pub struct BumpProxy {
    inner: Rc<UnsafeCell<BumpAllocator>>,
}

impl From<BumpAllocator> for BumpProxy {
    fn from(inner: BumpAllocator) -> Self {
        Self {
            inner: Rc::new(UnsafeCell::new(inner)),
        }
    }
}

impl Deref for BumpProxy {
    type Target = BumpAllocator;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

impl BumpProxy {
    pub unsafe fn get_inner(&mut self) -> &mut BumpAllocator {
        &mut *self.inner.get()
    }
}

#[derive(Debug)]
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

    ///# Safety
    ///
    ///Invalidates all outstanding pointers
    pub unsafe fn reset(&mut self, capacity: usize) {
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

    ///# Safety
    ///
    ///Invalidates all outstanding pointers
    pub unsafe fn clear(&mut self) {
        *self.head.get_mut() = 0;
    }

    /// # Safety
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

    /// # Safety
    ///
    /// Only pointers allocated by this instance are safe to free
    pub unsafe fn dealloc(&self, _p: NonNull<u8>, _l: Layout) {
        // noop
    }
}

impl Allocator for BumpAllocator {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        BumpAllocator::alloc(self, l)
    }

    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        BumpAllocator::dealloc(self, p, l)
    }
}
impl Allocator for BumpProxy {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        (*self.inner.get()).alloc(l)
    }

    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        (*self.inner.get()).dealloc(p, l)
    }
}
