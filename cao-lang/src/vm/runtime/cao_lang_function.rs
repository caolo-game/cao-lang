use std::ptr::NonNull;

use crate::{prelude::Handle, value::Value};

use super::cao_lang_object::CaoLangObject;

#[derive(Debug)]
pub struct CaoLangFunction {
    pub handle: Handle,
    pub arity: u32,
}

#[derive(Debug)]
pub struct CaoLangNativeFunction {
    pub handle: Handle,
}

pub struct CaoLangClosure {
    pub function: CaoLangFunction,
    pub upvalues: Vec<NonNull<CaoLangObject>>,
}

impl std::fmt::Debug for CaoLangClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaoLangClosure")
            .field("function", &self.function)
            .field(
                "upvalues",
                &self
                    .upvalues
                    .iter()
                    .map(|u| unsafe { (u, u.as_ref()) })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct CaoLangUpvalue {
    pub location: *mut Value,
    pub value: Value,
    pub next: *mut CaoLangObject,
}
