use crate::scalar::Scalar;
use thiserror::Error;

#[derive(Debug)]
pub struct ScalarStack {
    count: usize,
    buffer: Box<[Scalar]>,
}

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack was full")]
    Full,
}

impl ScalarStack {
    pub fn new(size: usize) -> Self {
        Self {
            count: 0,
            buffer: vec![Scalar::Null; size].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn push(&mut self, value: Scalar) -> Result<(), StackError> {
        if self.count + 1 < self.buffer.len() {
            self.buffer[self.count] = value;
            self.count += 1;
            Ok(())
        } else {
            Err(StackError::Full)
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.buffer[0] = Scalar::Null; // in case the stack is pop'ed when empty
    }

    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn pop(&mut self) -> Scalar {
        self.count = self.count.checked_sub(1).unwrap_or(0);
        std::mem::replace(&mut self.buffer[self.count], Scalar::Null)
    }

    pub fn as_slice(&self) -> &[Scalar] {
        &self.buffer[0..self.count]
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns Null if the stack is empty
    #[inline]
    pub fn last(&self) -> Scalar {
        if self.count > 0 {
            self.buffer[self.count - 1]
        } else {
            Scalar::Null
        }
    }
}
