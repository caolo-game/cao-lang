use std::{alloc::Layout, fmt::Debug, ptr::NonNull};

use crate::alloc::AllocProxy;

/// CaoLang Strings are immutable and UTF-8 encoded
pub struct CaoLangString {
    pub(crate) len: usize,
    pub(crate) ptr: NonNull<u8>,
    pub(crate) alloc: AllocProxy,
}

impl Debug for CaoLangString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.as_str();
        write!(f, "String: {str:?}")
    }
}

impl Drop for CaoLangString {
    fn drop(&mut self) {
        unsafe { self.alloc.dealloc(self.ptr.into(), Self::layout(self.len)) }
    }
}

impl CaoLangString {
    pub fn as_str(&self) -> &str {
        unsafe {
            let ptr = self.ptr;
            let len = self.len;
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr.as_ptr(), len as usize))
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Layout of a string with given length
    pub(crate) fn layout(len: usize) -> Layout {
        std::alloc::Layout::array::<char>(len).unwrap()
    }
}
