use crate::{prelude::Handle, value::Value};

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
    pub upvalues: Vec<CaoLangUpvalue>,
}

#[derive(Debug)]
pub struct CaoLangUpvalue {
    pub location: u32,
    pub value: Value,
}
