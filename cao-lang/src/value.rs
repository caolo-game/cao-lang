use crate::{vm::runtime::FieldTable, StrPointer};
use std::convert::{From, TryFrom};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    String(StrPointer),
    Object(*mut FieldTable),
    Integer(i64),
    Real(f64),
}

/// Intended for saving `Values` after the program has finished executing
///
/// ```
/// use cao_lang::prelude::*;
///
/// let mut vm = Vm::new(()).unwrap();
/// // init an object `val` with 1 entry {'pog': 42}
/// let mut obj = vm.init_table().unwrap();
/// let pog = vm.init_string("pog").unwrap();
/// unsafe { obj.as_mut() }
///     .insert(Value::String(pog), 42.into())
///     .unwrap();
/// let val = Value::Object(obj.as_ptr());
///
/// let owned = OwnedValue::from(val);
///
/// // (de)serialize the owned object...
///
/// // new vm instance
/// let mut vm = Vm::new(()).unwrap();
/// let loaded = vm.insert_value(&owned).unwrap();
///
/// # // check the contents
/// # let loaded_table = vm.get_table(loaded).unwrap();
/// # assert_eq!(loaded_table.len(), 1);
/// # for (k, v) in loaded_table.iter() {
/// #     let k = unsafe { k.as_str().unwrap() };
/// #     let v = v.as_int().unwrap();

/// #     assert_eq!(k, "pog");
/// #     assert_eq!(v, 42);
/// # }
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OwnedValue {
    Nil,
    String(String),
    Object(Vec<OwnedEntry>),
    Integer(i64),
    Real(f64),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnedEntry {
    pub key: OwnedValue,
    pub value: OwnedValue,
}

impl Default for OwnedValue {
    fn default() -> Self {
        OwnedValue::Nil
    }
}

impl From<Value> for OwnedValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Nil => Self::Nil,
            Value::String(_) => Self::String(unsafe { v.as_str() }.unwrap().to_owned()),
            Value::Object(ptr) => Self::Object({
                unsafe { &*ptr }
                    .iter()
                    .map(|(k, v)| OwnedEntry {
                        key: k.into(),
                        value: v.into(),
                    })
                    .collect()
            }),
            Value::Integer(x) => Self::Integer(x),
            Value::Real(x) => Self::Real(x),
        }
    }
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
            Value::String(i) => !i.0.is_null(),
            Value::Object(i) => !i.is_null(),
            Value::Integer(i) => i != 0,
            Value::Real(i) => i != 0.0,
            Value::Nil => false,
        }
    }

    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self, Value::Real(_))
    }

    #[inline]
    pub fn is_str(self) -> bool {
        matches!(self, Value::String(_))
    }

    /// # Safety
    ///
    /// Must be called with ptr obtained from a `string_literal` instruction, before the last `clear`!
    ///
    /// The Vm that allocated the string must still be in memory!
    ///
    /// # Return
    ///
    /// Returns `None` if the value is not a string, or points to an invalid string
    pub unsafe fn as_str<'a>(self) -> Option<&'a str> {
        match self {
            Value::String(StrPointer(ptr)) => {
                let len = *(ptr as *const u32);
                let ptr = ptr.add(4);
                std::str::from_utf8(std::slice::from_raw_parts(ptr, len as usize)).ok()
            }
            _ => None,
        }
    }

    /// # Safety
    ///
    /// Must be called with ptr obtained from a vm , before the last `clear`!
    ///
    /// The Vm that allocated the table must still be in memory!
    ///
    /// # Return
    ///
    /// Returns `None` if the value is not a table, or points to an invalid table
    pub unsafe fn as_table<'a>(self) -> Option<&'a FieldTable> {
        match self {
            Value::Object(table) => Some(&*table),
            _ => None,
        }
    }

    pub fn as_int(self) -> Option<i64> {
        match self {
            Value::Integer(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_real(self) -> Option<f64> {
        match self {
            Value::Real(x) => Some(x),
            _ => None,
        }
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
                Value::Real(
                    i64::try_from(self)
                        .map(|x| x as f64)
                        .or_else(f64::try_from)
                        .unwrap(),
                ),
                Value::Real(
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

impl TryFrom<Value> for StrPointer {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(ptr) => Ok(ptr),
            _ => Err(value),
        }
    }
}

impl From<Value> for bool {
    fn from(s: Value) -> Self {
        s.as_bool()
    }
}

impl TryFrom<Value> for *mut FieldTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Object(p) => Ok(p),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for &FieldTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            // NOTE:
            // this is still bad if p is dangling
            Value::Object(p) if !p.is_null() => Ok(unsafe { &*p }),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for &mut FieldTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            // NOTE:
            // this is still bad if p is dangling
            Value::Object(p) if !p.is_null() => Ok(unsafe { &mut *p }),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::String(i) => Ok(i.0 as i64),
            Value::Object(i) => Ok(i as i64),
            Value::Integer(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Real(i) => Ok(i),
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
            (Value::Real(a), Value::Real(b)) => Value::Real(a $op b),
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
