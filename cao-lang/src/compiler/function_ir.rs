use std::rc::Rc;

use super::{Card, ImportsIr, NameSpace};
use crate::VarName;

/// Intermediate function data
#[derive(Debug, Clone)]
pub struct FunctionIr {
    pub name: Box<str>,
    pub arguments: Box<[VarName]>,
    pub cards: Box<[Card]>,
    pub namespace: NameSpace,
    /// aliases this function sees
    ///
    /// TODO: we should compile modules instead of functions, and pass import per module...
    pub imports: Rc<ImportsIr>,
}
