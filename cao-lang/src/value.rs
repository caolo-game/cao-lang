use crate::prelude::{CaoLangTable, Handle};
use crate::vm::runtime::cao_lang_object::{CaoLangObject, CaoLangObjectBody};
use std::convert::{From, TryFrom};
use std::ops::{Add, Div, Mul, Sub};
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Object(NonNull<CaoLangObject>),
    Integer(i64),
    Real(f64),
    Function { hash: Handle, arity: u32 },
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let (this, other) = self.cast_match(*other);
        match (this, other) {
            (Value::Object(a), Value::Object(b)) => unsafe { a.as_ref().partial_cmp(b.as_ref()) },
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(&b),
            (Value::Real(a), Value::Real(b)) => a.partial_cmp(&b),
            _ => None,
        }
    }
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 0u8.hash(state),
            Value::Integer(i) => {
                i.hash(state);
            }
            Value::Real(f) => {
                f.to_bits().hash(state);
            }
            Value::Object(o) => unsafe {
                o.as_ref().hash(state);
            },
            Value::Function { hash, arity } => {
                hash.value().hash(state);
                arity.hash(state);
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Object(lhs), Value::Object(rhs)) => unsafe { lhs.as_ref().eq(rhs.as_ref()) },
            (Value::Integer(lhs), Value::Integer(rhs)) => lhs == rhs,
            (Value::Real(lhs), Value::Real(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Eq for Value {}

/// Intended for saving `Values` after the program has finished executing
///
/// ```
/// use cao_lang::prelude::*;
///
/// let mut vm = Vm::new(()).unwrap();
/// // init an object `val` with 1 entry {'pog': 42}
/// let mut obj = vm.init_table().unwrap();
/// let pog = vm.init_string("pog").unwrap();
/// obj.as_table_mut()
///     .unwrap()
///     .insert(Value::Object(pog.into_inner()), 42.into())
///     .unwrap();
/// let val = Value::Object(obj.into_inner());
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
    Table(Vec<OwnedEntry>),
    Integer(i64),
    Real(f64),
    Function { hash: Handle, arity: u32 },
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
            Value::Object(ptr) => unsafe {
                match &ptr.as_ref().body {
                    CaoLangObjectBody::Table(t) => Self::Table(
                        t.iter()
                            .map(|(k, v)| OwnedEntry {
                                key: (*k).into(),
                                value: (*v).into(),
                            })
                            .collect(),
                    ),
                    CaoLangObjectBody::String(s) => Self::String(s.as_str().to_owned()),
                }
            },
            Value::Integer(x) => Self::Integer(x),
            Value::Real(x) => Self::Real(x),
            Value::Function { hash, arity } => Self::Function { hash, arity },
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
            Value::Object(i) => unsafe { !i.as_ref().is_empty() },
            Value::Integer(i) => i != 0,
            Value::Real(i) => i != 0.0,
            Value::Nil => false,
            Value::Function { .. } => true,
        }
    }

    /// Typename of this value
    pub fn type_name(self) -> &'static str {
        match self {
            Value::Nil => "Nil",
            Value::Object(o) => unsafe { o.as_ref().type_name() },
            Value::Integer(_) => "Integer",
            Value::Real(_) => "Real",
            Value::Function { .. } => "Function",
        }
    }

    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self, Value::Real(_))
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
            Value::Object(o) => unsafe { o.as_ref().as_str() },
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
    pub unsafe fn as_table<'a>(self) -> Option<&'a CaoLangTable> {
        match self {
            Value::Object(table) => table.as_ref().as_table(),
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

impl TryFrom<Value> for &str {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => unsafe { o.as_ref().as_str().ok_or(value) },
            _ => Err(value),
        }
    }
}

impl From<Value> for bool {
    fn from(s: Value) -> Self {
        s.as_bool()
    }
}

impl TryFrom<Value> for *mut CaoLangTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Object(mut p) => unsafe {
                match p.as_mut().as_table_mut() {
                    Some(t) => Ok(t as *mut _),
                    _ => Err(v),
                }
            },
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for &CaoLangTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Object(p) => unsafe { p.as_ref().as_table().ok_or(v) },
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for &mut CaoLangTable {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Object(mut p) => unsafe { p.as_mut().as_table_mut().ok_or(v) },
            _ => Err(v),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
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

impl std::borrow::Borrow<str> for Value {
    fn borrow(&self) -> &str {
        match self {
            Value::Object(s) => unsafe { s.as_ref().as_str().unwrap_or("") },
            _ => "",
        }
    }
}

impl std::borrow::Borrow<i64> for Value {
    fn borrow(&self) -> &i64 {
        match self {
            Value::Integer(i) => i,
            _ => &(!0),
        }
    }
}

/// We can't implement TryFrom<Value> for Option<T>'s, you can use this wrapper in functions to
/// take an optional value
pub struct Nilable<T>(pub Option<T>);

impl<T> TryFrom<Value> for Nilable<T>
where
    T: TryFrom<Value>,
{
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Nil => Ok(Nilable(None)),
            _ => {
                let res = value.try_into().map_err(|_| value)?;
                Ok(Nilable(Some(res)))
            }
        }
    }
}
