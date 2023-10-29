use std::rc::Rc;

use super::{Card, ImportsIr, NameSpace};
use crate::VarName;

/// Intermediate function data
#[derive(Debug, Clone)]
pub struct FunctionIr {
    /// function's index in the context of its namespace
    pub function_index: usize,
    pub name: Box<str>,
    pub arguments: Box<[VarName]>,
    pub cards: Box<[Card]>,
    pub namespace: NameSpace,
    /// aliases this function sees
    ///
    /// TODO: we should compile modules instead of functions, and pass import per module...
    pub imports: Rc<ImportsIr>,
    pub handle: crate::prelude::Handle,
}

impl FunctionIr {
    pub fn full_name(&self) -> String {
        if self.namespace.is_empty() {
            return self.name.to_string();
        }
        format!("{}.{}", self.namespace.join("."), self.name)
    }
}
