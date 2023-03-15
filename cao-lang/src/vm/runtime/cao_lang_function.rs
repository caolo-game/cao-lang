use crate::prelude::Handle;

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
}
