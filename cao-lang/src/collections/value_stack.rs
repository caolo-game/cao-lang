//! Stack containing only cao-lang Values
//! Because Values can express `nil` values we use them instead of optionals
//!
use crate::value::Value;
use thiserror::Error;

#[derive(Debug)]
pub struct ValueStack {
    count: usize,
    data: Box<[Value]>,
}

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack is full")]
    Full,
    #[error("Index out of bounds: capacity: {capacity} index: {index}")]
    OutOfBounds { capacity: usize, index: usize },
}
impl std::fmt::Display for ValueStack {
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

impl ValueStack {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        Self {
            count: 0,
            data: vec![Value::Nil; size].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Value] {
        &self.data[0..self.count]
    }

    #[inline]
    pub fn push<T: Into<Value>>(&mut self, value: T) -> Result<(), StackError> {
        if self.count + 1 < self.data.len() {
            self.data[self.count] = value.into();
            self.count += 1;
            Ok(())
        } else {
            Err(StackError::Full)
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.data[0] = Value::Nil; // in case the stack is popped when empty
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns Nil if the stack is empty
    #[inline]
    pub fn pop(&mut self) -> Value {
        let count = self.count.saturating_sub(1);
        let s = self.data[count];
        self.count = count;
        self.data[self.count] = Value::Nil;
        s
    }

    /// Pop value, treating offset as the 0 position
    ///
    /// ```
    /// use cao_lang::collections::value_stack::ValueStack;
    /// use cao_lang::prelude::Value;
    ///
    /// let mut stack = ValueStack::new(4);
    /// stack.push(Value::Integer(42));
    /// let res = stack.pop_w_offset(1);
    /// assert_eq!(res, Value::Nil);
    /// let res = stack.pop();
    /// assert_eq!(res, Value::Integer(42));
    /// ```
    ///
    pub fn pop_w_offset(&mut self, offset: usize) -> Value {
        if self.count <= offset {
            return Value::Nil;
        }
        self.pop()
    }

    /// Sets a value
    /// Only previous values may be set
    ///
    /// Returns the old value
    pub fn set(&mut self, index: usize, value: Value) -> Result<Value, StackError> {
        if index > self.count {
            return Err(StackError::OutOfBounds {
                capacity: self.count,
                index,
            });
        }
        if index == self.count {
            self.push(value)?;
            Ok(Value::Nil)
        } else {
            let old = std::mem::replace(&mut self.data[index], value);
            Ok(old)
        }
    }

    pub fn get(&mut self, index: usize) -> Value {
        if index >= self.count {
            return Value::Nil;
        }
        self.data[index]
    }

    /// Returns the very first item
    pub fn clear_until(&mut self, index: usize) -> Value {
        let res = self.last();
        while self.count > index {
            self.count = self.count.saturating_sub(1);
            self.data[self.count] = Value::Nil;
        }
        res
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns Null if the stack is empty
    #[inline]
    pub fn last(&self) -> Value {
        if self.count > 0 {
            self.data[self.count - 1]
        } else {
            Value::Nil
        }
    }

    /// Returns Null if the index is out of bounds
    #[inline]
    pub fn peek_last(&self, n: usize) -> Value {
        if self.count > n {
            self.data[self.count - n - 1]
        } else {
            Value::Nil
        }
    }
}
