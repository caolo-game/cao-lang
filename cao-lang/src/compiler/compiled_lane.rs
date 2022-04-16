use super::Card;
use crate::VarName;

/// Cao-lang functions
#[derive(Debug, Clone, Default)]
pub struct CompiledLane {
    pub name: String,
    pub namespace: smallvec::SmallVec<[String; 16]>,
    pub arguments: Vec<VarName>,
    pub cards: Vec<Card>,
}
