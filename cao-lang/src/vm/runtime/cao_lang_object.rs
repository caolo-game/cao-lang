use std::ptr::NonNull;

use crate::value::Value;

use super::{
    cao_lang_function::{CaoLangFunction, CaoLangNativeFunction},
    cao_lang_string::CaoLangString,
    cao_lang_table::CaoLangTable,
};

// note Gray is not actually useful for now, but it might come in handy if we want to do finalizers
#[derive(Debug, Clone, Copy)]
pub enum GcMarker {
    /// Unprocessed
    White,
    /// Visited
    Gray,
    /// Done
    Black,
    /// This object can not be collected
    Protected,
}

pub struct CaoLangObject {
    pub marker: GcMarker,
    pub body: CaoLangObjectBody,
}

#[derive(Debug)]
pub enum CaoLangObjectBody {
    Table(CaoLangTable),
    String(CaoLangString),
    Function(CaoLangFunction),
    NativeFunction(CaoLangNativeFunction),
}

/// RAII style guard that ensures that an object survives the GC
/// Useful for native function that allocate multiple objects, potentially triggering GC
pub struct ObjectGcGuard(pub(crate) NonNull<CaoLangObject>);

impl std::ops::Deref for ObjectGcGuard {
    type Target = CaoLangObject;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl std::ops::DerefMut for ObjectGcGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl Drop for ObjectGcGuard {
    fn drop(&mut self) {
        unsafe {
            self.0.as_mut().marker = GcMarker::White;
        }
    }
}

impl ObjectGcGuard {
    pub fn new(mut obj: NonNull<CaoLangObject>) -> Self {
        unsafe {
            obj.as_mut().marker = GcMarker::Protected;
        }
        Self(obj)
    }

    pub fn into_inner(self) -> NonNull<CaoLangObject> {
        self.0
    }
}

impl Into<Value> for ObjectGcGuard {
    fn into(self) -> Value {
        Value::Object(self.0)
    }
}

impl CaoLangObject {
    pub fn type_name(&self) -> &'static str {
        match &self.body {
            CaoLangObjectBody::Table(_) => "Table",
            CaoLangObjectBody::String(_) => "String",
            CaoLangObjectBody::Function(_) => "Function",
            CaoLangObjectBody::NativeFunction(_) => "NativeFunction",
        }
    }

    pub fn as_table(&self) -> Option<&CaoLangTable> {
        match &self.body {
            CaoLangObjectBody::Table(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_table_mut(&mut self) -> Option<&mut CaoLangTable> {
        match &mut self.body {
            CaoLangObjectBody::Table(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.body {
            CaoLangObjectBody::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&CaoLangFunction> {
        match &self.body {
            CaoLangObjectBody::Function(f) => Some(f),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match &self.body {
            CaoLangObjectBody::Table(t) => t.len(),
            CaoLangObjectBody::String(s) => s.len(),
            CaoLangObjectBody::Function(_) => 0,
            CaoLangObjectBody::NativeFunction(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.body {
            CaoLangObjectBody::Table(_) | CaoLangObjectBody::String(_) => self.len() == 0,
            CaoLangObjectBody::Function(_) | CaoLangObjectBody::NativeFunction(_) => false,
        }
    }
}

impl std::hash::Hash for CaoLangObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.body {
            CaoLangObjectBody::Table(o) => {
                for (k, v) in o.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            CaoLangObjectBody::String(s) => {
                s.as_str().hash(state);
            }
            CaoLangObjectBody::Function(f) => {
                f.handle.value().hash(state);
                f.arity.hash(state);
            }
            CaoLangObjectBody::NativeFunction(f) => f.handle.value().hash(state),
        }
    }
}

impl PartialEq for CaoLangObject {
    fn eq(&self, other: &Self) -> bool {
        match (&self.body, &other.body) {
            (CaoLangObjectBody::Table(lhs), CaoLangObjectBody::Table(rhs)) => {
                if lhs.len() != rhs.len() {
                    return false;
                }
                for ((kl, vl), (kr, vr)) in lhs.iter().zip(rhs.iter()) {
                    if kl != kr || vl != vr {
                        return false;
                    }
                }
                true
            }
            (CaoLangObjectBody::String(lhs), CaoLangObjectBody::String(rhs)) => {
                lhs.as_str().eq(rhs.as_str())
            }
            _ => false,
        }
    }
}

impl PartialOrd for CaoLangObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.eq(other)
            .then_some(std::cmp::Ordering::Equal)
            .or_else(|| {
                // equal len but non-eq objects should not return Equal
                let res = self.len().cmp(&other.len());
                match res {
                    std::cmp::Ordering::Equal => None,
                    _ => Some(res),
                }
            })
    }
}
