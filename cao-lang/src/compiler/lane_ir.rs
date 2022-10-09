use std::{collections::BTreeMap, rc::Rc};

use super::{Card, CardId, Imports, NameSpace};
use crate::VarName;

/// Intermediate lane data
#[derive(Debug, Clone, Default)]
pub struct LaneIr {
    pub name: Box<str>,
    pub arguments: Box<[VarName]>,
    pub cards: Box<[CardId]>,
    pub namespace: NameSpace,
    /// aliases this lane sees
    ///
    /// TODO: we should compile modules instead of lanes, and pass import per module...
    pub imports: Rc<Imports>,
    // TODO: use a map reference?
    pub card_impls: BTreeMap<CardId, Card>,
}
