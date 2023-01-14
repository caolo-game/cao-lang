use tracing::debug;

use crate::vm::runtime::RuntimeData;

use super::{AllocError, Allocator};
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::UnsafeCell,
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

/// # Safety
///
/// Note that CaoLangAllocator is NOT thread-safe!
#[derive(Debug, Clone)]
pub struct AllocProxy {
    inner: Rc<UnsafeCell<CaoLangAllocator>>,
    _m: PhantomData<*mut CaoLangAllocator>,
}

impl From<CaoLangAllocator> for AllocProxy {
    fn from(inner: CaoLangAllocator) -> Self {
        Self {
            inner: Rc::new(UnsafeCell::new(inner)),
            _m: PhantomData,
        }
    }
}

impl Deref for AllocProxy {
    type Target = CaoLangAllocator;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

impl AllocProxy {
    pub unsafe fn get_inner(&mut self) -> &mut CaoLangAllocator {
        &mut *self.inner.get()
    }
}

#[derive(Debug)]
pub struct CaoLangAllocator {
    pub runtime: *mut RuntimeData,
    pub allocated: AtomicUsize,
    pub next_gc: AtomicUsize,
    pub limit: AtomicUsize,
}

impl CaoLangAllocator {
    pub fn new(vm: *mut RuntimeData, limit: usize) -> Self {
        Self {
            runtime: vm,
            allocated: AtomicUsize::new(0),
            next_gc: AtomicUsize::new((limit / 4).max(16)),
            limit: AtomicUsize::new(limit),
        }
    }

    /// # Safety
    /// `alloc` is not thread safe. It is on the caller to ensure that only a single thread uses
    /// the allocator at a time
    pub unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        let s = l.size() + l.align();
        let allocated = s + self.allocated.fetch_add(s, Ordering::Relaxed);
        if allocated > self.limit.load(Ordering::Relaxed) {
            return Err(AllocError::OutOfMemory);
        }
        if allocated > self.next_gc.load(Ordering::Relaxed) {
            self.next_gc.store(allocated * 2, Ordering::Relaxed);
            unsafe {
                (*self.runtime).gc();
            }
            debug!(
                "GC done. Allocated before: {allocated}. Allocated now: {}",
                self.allocated.load(Ordering::Relaxed)
            );
        }
        let ptr = alloc(l);
        Ok(NonNull::new(ptr).unwrap())
    }

    /// # Safety
    ///
    /// Only pointers allocated by this instance are safe to free
    pub unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        let s = l.size() + l.align();
        self.allocated.fetch_sub(s, Ordering::Relaxed);
        dealloc(p.as_ptr(), l);
    }
}

impl Allocator for CaoLangAllocator {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        CaoLangAllocator::alloc(self, l)
    }

    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        CaoLangAllocator::dealloc(self, p, l)
    }
}
impl Allocator for AllocProxy {
    unsafe fn alloc(&self, l: Layout) -> Result<NonNull<u8>, AllocError> {
        (*self.inner.get()).alloc(l)
    }

    unsafe fn dealloc(&self, p: NonNull<u8>, l: Layout) {
        (*self.inner.get()).dealloc(p, l)
    }
}
