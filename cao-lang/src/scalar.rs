use crate::{traits::AutoByteEncodeProperties, Pointer};
use std::convert::{From, TryFrom};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Scalar {
    Null,
    Pointer(Pointer),
    Integer(i32),
    Floating(f32),
}

impl Default for Scalar {
    fn default() -> Self {
        Scalar::Null
    }
}

impl Scalar {
    #[inline]
    pub fn as_bool(self) -> bool {
        match self {
            Scalar::Pointer(Pointer(i)) => i != 0,
            Scalar::Integer(i) => i != 0,
            Scalar::Floating(i) => i != 0.0,
            Scalar::Null => false,
        }
    }

    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self, Scalar::Floating(_))
    }

    #[inline]
    pub fn is_ptr(self) -> bool {
        matches!(self, Scalar::Pointer(_))
    }

    #[inline]
    pub fn is_integer(self) -> bool {
        matches!(self, Scalar::Integer(_))
    }

    /// If either is a float cast both to a floating point number, else cast both to Integer
    fn cast_match(self, other: Self) -> (Self, Self) {
        if self.is_float() || other.is_float() {
            return (
                Scalar::Floating(
                    i32::try_from(self)
                        .map(|x| x as f32)
                        .or_else(f32::try_from)
                        .unwrap(),
                ),
                Scalar::Floating(
                    i32::try_from(other)
                        .map(|x| x as f32)
                        .or_else(f32::try_from)
                        .unwrap(),
                ),
            );
        }
        let a = i32::try_from(self).unwrap();
        let b = i32::try_from(other).unwrap();

        (Scalar::Integer(a), Scalar::Integer(b))
    }
}

impl AutoByteEncodeProperties for Scalar {}

impl From<Scalar> for bool {
    fn from(s: Scalar) -> Self {
        s.as_bool()
    }
}

impl TryFrom<Scalar> for Pointer {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Pointer(p) => Ok(p),
            _ => Err(v),
        }
    }
}

impl TryFrom<Scalar> for i32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Pointer(Pointer(i)) => Ok(i as i32),
            Scalar::Integer(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Scalar> for f32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Floating(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl From<i32> for Scalar {
    fn from(i: i32) -> Self {
        Scalar::Integer(i)
    }
}

impl From<bool> for Scalar {
    fn from(i: bool) -> Self {
        Scalar::Integer(i as i32)
    }
}

macro_rules! binary_op {
    ($a: expr, $b: expr, $op: tt) => {
        {
        let (a, b) = $a.cast_match($b);
        match (a, b) {
            (Scalar::Integer(a), Scalar::Integer(b)) => {
                #[allow(clippy::suspicious_arithmetic_impl)]
                if $a.is_ptr() || $b.is_ptr() {
                    Scalar::Pointer(Pointer((a $op b) as u32))
                } else {
                    Scalar::Integer(a $op b)
                }
            }
            (Scalar::Floating(a), Scalar::Floating(b)) => Scalar::Floating(a $op b),
            _ => unreachable!(),
        }
        }
    }
}

impl Add for Scalar {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        binary_op!(self, other, +)
    }
}

impl Sub for Scalar {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        binary_op!(self, other, -)
    }
}

impl Mul for Scalar {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        binary_op!(self, other, *)
    }
}

impl Div for Scalar {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        binary_op!(self, other, /)
    }
}
