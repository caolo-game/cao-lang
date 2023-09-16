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

#[derive(Debug)]
pub struct CaoLangClosure {
    pub function: CaoLangFunction,
    pub upvalues: Vec<NonNull<CaoLangObject>>,
}

#[derive(Debug)]
pub struct CaoLangUpvalue {
    pub location: *mut Value,
    pub value: Value,
}
