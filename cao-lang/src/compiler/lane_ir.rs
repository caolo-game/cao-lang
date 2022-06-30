use super::{Card, NameSpace};
use crate::VarName;

/// Intermediate lane data
#[derive(Debug, Clone, Default)]
pub struct LaneIr {
    pub name: Box<str>,
    pub arguments: Box<[VarName]>,
    pub cards: Box<[Card]>,
    pub namespace: NameSpace,
}
