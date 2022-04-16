use super::{Card, NameSpace};
use crate::VarName;

/// Cao-lang functions
#[derive(Debug, Clone, Default)]
pub struct CompiledLane {
    pub name: String,
    pub namespace: NameSpace,
    pub arguments: Vec<VarName>,
    pub cards: Vec<Card>,
}
