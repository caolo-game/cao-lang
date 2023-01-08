use std::rc::Rc;

use super::{Card, ImportsIr, NameSpace};
use crate::VarName;

/// Intermediate lane data
#[derive(Debug, Clone)]
pub struct LaneIr {
    pub name: Box<str>,
    pub arguments: Box<[VarName]>,
    pub cards: Box<[Card]>,
    pub namespace: NameSpace,
    pub lane_id: usize,
    /// aliases this lane sees
    ///
    /// TODO: we should compile modules instead of lanes, and pass import per module...
    pub imports: Rc<ImportsIr>,
}
