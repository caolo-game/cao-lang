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

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack is full")]
    Full,
    #[error("Index out of bounds: capacity: {capacity} index: {index}")]
    OutOfBounds { capacity: usize, index: usize },
}
impl std::fmt::Display for ScalarStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            return write!(f, "[]");
        }
        write!(f, "[ {:?}", self.data[0])?;
        for i in 1..self.count {
            write!(f, ", {:?}", &self.data[i])?;
        }
        write!(f, " ]")
    }
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

    /// Pop value, treating offset as the 0 position
    ///
    /// ```
    /// use cao_lang::collections::scalar_stack::ScalarStack;
    /// use cao_lang::prelude::Scalar;
    ///
    /// let mut stack = ScalarStack::new(4);
    /// stack.push(Scalar::Integer(42));
    /// let res = stack.pop_w_offset(1);
    /// assert_eq!(res, Scalar::Null);
    /// let res = stack.pop();
    /// assert_eq!(res, Scalar::Integer(42));
    /// ```
    ///
    pub fn pop_w_offset(&mut self, offset: usize) -> Scalar {
        if self.count <= offset {
            return Scalar::Null;
        }
        self.pop()
    }

    /// Sets a value
    /// Only previous values may be set
    ///
    /// Returns the old value
    pub fn set(&mut self, index: usize, value: Scalar) -> Result<Scalar, StackError> {
        if index > self.count {
            return Err(StackError::OutOfBounds {
                capacity: self.count,
                index,
            });
        }
        if index == self.count {
            self.push(value)?;
            Ok(Scalar::Null)
        } else {
            let old = std::mem::replace(&mut self.data[index], value);
            Ok(old)
        }
    }

    pub fn get(&mut self, index: usize) -> Scalar {
        if index >= self.count {
            return Scalar::Null;
        }
        self.data[index]
    }

    /// Returns the very first item
    pub fn clear_until(&mut self, index: usize) -> Scalar {
        let res = self.last();
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
