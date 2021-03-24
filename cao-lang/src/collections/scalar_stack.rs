//! Stack containing only cao-lang Scalars
//! Because Scalars can express `null` values we use them instead of optionals
//!
use crate::scalar::Scalar;
use thiserror::Error;

#[derive(Debug)]
pub struct ScalarStack {
    count: usize,
    data: Box<[StackEntry]>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Sentinel;

#[derive(Debug, Clone, Copy)]
pub enum StackEntry {
    /// Sentinels split the stack into regions
    Sentinel,
    Scalar(Scalar),
}

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
            data: vec![StackEntry::Sentinel; size].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[StackEntry] {
        &self.data[0..self.count]
    }

    #[inline]
    pub fn push_sentinel(&mut self) -> Result<(), StackError> {
        self._push(StackEntry::Sentinel)
    }

    fn _push(&mut self, value: StackEntry) -> Result<(), StackError> {
        if self.count + 1 < self.data.len() {
            self.data[self.count] = value;
            self.count += 1;
            Ok(())
        } else {
            Err(StackError::Full)
        }
    }

    #[inline]
    pub fn push(&mut self, value: Scalar) -> Result<(), StackError> {
        self._push(StackEntry::Scalar(value))
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.data[0] = StackEntry::Sentinel; // in case the stack is popped when empty
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns Scalar::Null if the stack is empty
    #[inline]
    pub fn pop(&mut self) -> Scalar {
        let count = self.count.saturating_sub(1);
        // if we hit a sentinel we don't actually return a value
        match self.data[count] {
            StackEntry::Sentinel => Scalar::Null,
            StackEntry::Scalar(s) => {
                self.count = count;
                self.data[self.count] = StackEntry::Sentinel;
                s
            }
        }
    }

    /// pop all values until a sentinel is hit
    ///
    /// Returns the very frist item
    pub fn clear_until_sentinel(&mut self) -> Scalar {
        let res = self.pop();
        let mut count = self.count.saturating_sub(1);
        while count > 0 && matches!(self.data[count], StackEntry::Scalar(_)) {
            self.data[count] = StackEntry::Sentinel;
            count -= 1;
        }
        self.data[count] = StackEntry::Sentinel;
        self.count = count;
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
            match self.data[self.count - 1] {
                StackEntry::Sentinel => Scalar::Null,
                StackEntry::Scalar(s) => s,
            }
        } else {
            Scalar::Null
        }
    }
}
