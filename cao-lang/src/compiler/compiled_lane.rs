use super::{Card, NameSpace};
use crate::VarName;

#[derive(Debug, Clone, Default)]
pub struct CompiledLane {
    pub name: String,
    pub arguments: Vec<VarName>,
    pub cards: Vec<Card>,
    pub namespace: NameSpace,
}
