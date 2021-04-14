use crate::Pointer;
use std::convert::{From, TryFrom};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub enum Value {
    Nil,
    String(Pointer),
    Object(Pointer),
    Integer(i64),
    Floating(f64),
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl Value {
    #[inline]
    pub fn as_bool(self) -> bool {
        match self {
            Value::String(Pointer(i)) | Value::Object(Pointer(i)) => !i.is_null(),
            Value::Integer(i) => i != 0,
            Value::Floating(i) => i != 0.0,
            Value::Nil => false,
        }
    }

    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self, Value::Floating(_))
    }

    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline]
    pub fn is_obj(self) -> bool {
        matches!(self, Value::Object(_))
    }

    #[inline]
    pub fn is_integer(self) -> bool {
        matches!(self, Value::Integer(_))
    }

    #[inline]
    pub fn is_null(self) -> bool {
        matches!(self, Value::Nil)
    }

    /// If either is a float cast both to a floating point number, else cast both to Integer
    fn cast_match(self, other: Self) -> (Self, Self) {
        if self.is_float() || other.is_float() {
            return (
                Value::Floating(
                    i64::try_from(self)
                        .map(|x| x as f64)
                        .or_else(f64::try_from)
                        .unwrap(),
                ),
                Value::Floating(
                    i64::try_from(other)
                        .map(|x| x as f64)
                        .or_else(f64::try_from)
                        .unwrap(),
                ),
            );
        }
        if self.is_null() || other.is_null() {
            return (Value::Nil, Value::Nil);
        }

        let a = i64::try_from(self).unwrap();
        let b = i64::try_from(other).unwrap();

        (Value::Integer(a), Value::Integer(b))
    }
}

impl From<Value> for bool {
    fn from(s: Value) -> Self {
        s.as_bool()
    }
}

impl TryFrom<Value> for Pointer {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::String(p) | Value::Object(p) => Ok(p),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Object(Pointer(i)) => Ok(i as i64),
            Value::Integer(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Floating(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<bool> for Value {
    fn from(i: bool) -> Self {
        Value::Integer(i as i64)
    }
}

macro_rules! binary_op {
    ($a: expr, $b: expr, $op: tt) => {
        {
        let (a, b) = $a.cast_match($b);
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => {
                    Value::Integer(a $op b)
            }
            (Value::Floating(a), Value::Floating(b)) => Value::Floating(a $op b),
            _ => Value::Nil
        }
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        binary_op!(self, other, +)
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        binary_op!(self, other, -)
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        binary_op!(self, other, *)
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        binary_op!(self, other, /)
    }
}
