use std::fmt::Display;

use super::{LaneNode, NodeId};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub struct CompilationError {
    pub payload: CompilationErrorPayload,
    /// (lane, card)
    pub loc: Option<(LaneNode, i32)>,
}

impl CompilationError {
    pub fn with_loc(payload: CompilationErrorPayload, lane_id: LaneNode, card_id: i32) -> Self {
        Self {
            payload,
            loc: Some((lane_id, card_id)),
        }
    }
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = self.loc.as_ref() {
            write!(
                f,
                "CompilationError: [Lane: {} Card: {}], Error: {}",
                loc.0, loc.1, self.payload
            )
        } else {
            write!(f, "{}", self.payload)
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CompilationErrorPayload {
    #[error("The requested functionality ({0}) is not yet implemented")]
    Unimplemented(&'static str),

    #[error("Program was empty")]
    EmptyProgram,

    #[error("Number of lanes may not be larger than 2^16 - 1 = 65535")]
    TooManyLanes,
    #[error("Lanes {0} has too many cards. Number of cards in a lane may not be larger than 2^16 - 1 = 65535")]
    TooManyCards(usize),

    #[error("Lane names must be unique. Found duplicated name: {0}")]
    DuplicateName(String),

    #[error("SubProgram: [{0}] was not found")]
    MissingSubProgram(String),

    #[error("Program references node [{0:?}] but it was not found")]
    MissingNode(NodeId),

    #[error("Jumping to {dst} can not be performed\n{msg:?}")]
    InvalidJump { dst: LaneNode, msg: Option<String> },

    #[error("Internal failure during compilation")]
    InternalError,

    #[error("Too many locals in scope")]
    TooManyLocals,

    #[error("Variable name {0} can not be used")]
    BadVariableName(String),

    #[error("Variable name can't be empty")]
    EmptyVariable,
}
