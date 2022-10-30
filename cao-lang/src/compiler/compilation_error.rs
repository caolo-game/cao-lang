use std::fmt::Display;

use super::{CardIndex, LaneNode};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub struct CompilationError {
    pub payload: CompilationErrorPayload,
    pub loc: Option<CardIndex>,
}

impl CompilationError {
    pub fn with_loc(payload: CompilationErrorPayload, index: CardIndex) -> Self {
        Self {
            payload,
            loc: Some(index),
        }
    }
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = self.loc.as_ref() {
            write!(f, "CompilationError: [{}], Error: {}", loc, self.payload)
        } else {
            write!(f, "{}", self.payload)
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CompilationErrorPayload {
    #[error("The requested functionality ({0}) is not yet implemented")]
    Unimplemented(&'static str),

    #[error("Entrypoint not found")]
    NoMain,

    #[error("Program was empty")]
    EmptyProgram,

    #[error("Can not handle that many lanes")]
    TooManyLanes,

    #[error("Lanes {0} has too many cards. Number of cards in a lane may not be larger than 2^16 - 1 = 65535")]
    TooManyCards(usize),

    #[error("Lane names must be unique. Found duplicated name: {0}")]
    DuplicateName(String),

    #[error("SubProgram: [{0}] was not found")]
    MissingSubProgram(String),

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

    #[error("{0:?} is not a valid name for a Lane")]
    BadLaneName(String),

    #[error("Recursion limit ({0}) reached")]
    RecursionLimitReached(u32),

    #[error("Import '{0}' is not valid")]
    BadImport(String),

    #[error("Too many `super.` calls.")]
    SuperLimitReached,
}
