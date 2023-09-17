use crate::prelude::CaoLangTable;
use crate::vm::runtime::cao_lang_object::{CaoLangObject, CaoLangObjectBody};
use std::convert::{From, TryFrom};
use std::ops::{Add, Div, Mul, Sub};
use std::ptr::NonNull;

#[derive(Clone, Copy)]
pub enum Value {
    Nil,
    Object(NonNull<CaoLangObject>),
    Integer(i64),
    Real(f64),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Object(arg0) => f
                .debug_tuple("Object")
                .field(&arg0)
                .field(unsafe { arg0.as_ref() })
                .finish(),
            Self::Integer(arg0) => f.debug_tuple("Integer").field(arg0).finish(),
            Self::Real(arg0) => f.debug_tuple("Real").field(arg0).finish(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let (this, other) = self.try_cast_match(*other);
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
///     .insert(Value::Object(pog.into_inner()), 42)
///     .unwrap();
/// let val = Value::Object(obj.into_inner());
///
/// let owned = OwnedValue::try_from(val).unwrap();
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

impl TryFrom<Value> for OwnedValue {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let res = match v {
            Value::Nil => Self::Nil,
            Value::Object(ptr) => unsafe {
                match &ptr.as_ref().body {
                    CaoLangObjectBody::Table(t) => {
                        let mut entries = Vec::with_capacity(t.len());
                        for (k, v) in t.iter() {
                            entries.push(OwnedEntry {
                                key: (*k).try_into()?,
                                value: (*v).try_into()?,
                            });
                        }
                        Self::Table(entries)
                    }
                    CaoLangObjectBody::String(s) => Self::String(s.as_str().to_owned()),
                    CaoLangObjectBody::Function(_)
                    | CaoLangObjectBody::Closure(_)
                    | CaoLangObjectBody::NativeFunction(_)
                    | CaoLangObjectBody::Upvalue(_) => {
                        return Err(v);
                    }
                }
            },
            Value::Integer(x) => Self::Integer(x),
            Value::Real(x) => Self::Real(x),
        };
        Ok(res)
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
        }
    }

    /// Typename of this value
    pub fn type_name(self) -> &'static str {
        match self {
            Value::Nil => "Nil",
            Value::Object(o) => unsafe { o.as_ref().type_name() },
            Value::Integer(_) => "Integer",
            Value::Real(_) => "Real",
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

    /// return the original pair if casting can't be performed
    fn try_cast_match(self, other: Self) -> (Self, Self) {
        if self.is_float() || other.is_float() {
            if let Ok(a) = f64::try_from(self) {
                if let Ok(b) = f64::try_from(other) {
                    return (Value::Real(a), Value::Real(b));
                }
            }
        }
        if self.is_integer() || other.is_integer() {
            if let Ok(a) = i64::try_from(self) {
                if let Ok(b) = i64::try_from(other) {
                    return (Value::Integer(a), Value::Integer(b));
                }
            }
        }
        (self, other)
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
            Value::Real(r) => Ok(r as i64),
            Value::Object(o) => Ok(unsafe { o.as_ref().len() as i64 }),
            Value::Nil => Ok(0),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Real(i) => Ok(i),
            Value::Integer(i) => Ok(i as f64),
            Value::Object(o) => Ok(unsafe { o.as_ref().len() as f64 }),
            Value::Nil => Ok(0.0),
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
        let (a, b) = $a.try_cast_match($b);
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
        let (a, b) = self.try_cast_match(other);
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Value::Real(a as f64 / b as f64),
            (Value::Real(a), Value::Real(b)) => Value::Real(a / b),
            _ => Value::Nil,
        }
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
