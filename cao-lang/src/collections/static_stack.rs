use std::{
    mem::MaybeUninit,
    ptr::{self, drop_in_place},
};
use thiserror::Error;

pub struct Stack<T> {
    head: usize,
    capacity: usize,
    storage: Box<[MaybeUninit<T>]>,
}

#[derive(Clone, Debug, Error)]
pub enum StackError {
    #[error("Stack is full")]
    Full,
}

impl<T> Stack<T> {
    pub fn new(capacity: usize) -> Self {
        let mut storage = Vec::new();
        storage.resize_with(capacity, MaybeUninit::uninit);
        let storage = storage.into_boxed_slice();
        Self {
            head: 0,
            capacity,
            storage,
        }
    }

    pub fn len(&self) -> usize {
        self.head
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let ptr: *const MaybeUninit<T> = self.storage.as_ptr();
        (0..self.head).map(move |i| unsafe { &*(*ptr.add(i)).as_ptr() })
    }

    pub fn push(&mut self, val: T) -> Result<(), StackError> {
        if self.head >= self.capacity {
            return Err(StackError::Full);
        }
        unsafe {
            ptr::write(self.storage.get_unchecked_mut(self.head).as_mut_ptr(), val);
        }
        self.head += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        (self.head > 0).then(|| {
            self.head -= 1;
            unsafe { ptr::read(self.storage.get_unchecked(self.head).as_ptr()) }
        })
    }

    pub fn clear(&mut self) {
        if std::mem::needs_drop::<T>() {
            for i in 0..self.head {
                unsafe { drop_in_place(self.storage.get_unchecked_mut(i).as_mut_ptr()) }
            }
        }
        self.head = 0;
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_on_clear() {
        let mut drops = Box::pin(0);

        struct Foo(*mut u32);
        impl Drop for Foo {
            fn drop(&mut self) {
                assert_ne!(self.0 as *const _, std::ptr::null());
                unsafe {
                    *self.0 += 1;
                }
            }
        }

        let mut stack = Stack::new(5);
        for _ in 0..5 {
            stack.push(Foo(drops.as_mut().get_mut())).unwrap();
        }

        assert_eq!(*drops, 0);
        assert_eq!(stack.len(), 5);

        stack.clear();

        assert_eq!(*drops, 5);
        assert_eq!(stack.len(), 0);
    }
}
