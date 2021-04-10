//! Stack containing only cao-lang Scalars
//! Because Scalars can express `null` values we use them instead of optionals
//!
use crate::scalar::Scalar;
use thiserror::Error;

#[derive(Debug)]
pub struct ScalarStack {
    count: usize,
    data: Box<[Scalar]>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Sentinel;

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack is full")]
    Full,
}

impl ScalarStack {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        Self {
            count: 0,
            data: vec![Scalar::Null; size].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Scalar] {
        &self.data[0..self.count]
    }

    #[inline]
    pub fn push(&mut self, value: Scalar) -> Result<(), StackError> {
        if self.count + 1 < self.data.len() {
            self.data[self.count] = value;
            self.count += 1;
            Ok(())
        } else {
            Err(StackError::Full)
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.data[0] = Scalar::Null; // in case the stack is popped when empty
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns Scalar::Null if the stack is empty
    #[inline]
    pub fn pop(&mut self) -> Scalar {
        let count = self.count.saturating_sub(1);
        let s = self.data[count];
        self.count = count;
        self.data[self.count] = Scalar::Null;
        s
    }

    /// Returns the very first item
    pub fn clear_until(&mut self, index: usize) -> Scalar {
        let res = self.pop();
        while self.count > index {
            self.count = self.count.saturating_sub(1);
            self.data[self.count] = Scalar::Null;
        }
        res
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns Null if the stack is empty
    #[inline]
    pub fn last(&self) -> Scalar {
        if self.count > 0 {
            self.data[self.count - 1]
        } else {
            Scalar::Null
        }
    }
}
