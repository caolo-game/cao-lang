use super::NodeId;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CompilationError {
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

    #[error("Jumping from {src:?} to {dst} can not be performed\n{msg:?}")]
    InvalidJump {
        src: NodeId,
        dst: String,
        msg: Option<String>,
    },

    #[error("Internal failure during compilation")]
    InternalError,

    #[error("Too many locals in scope")]
    TooManyLocals,

    #[error("Variable name {0} can not be used")]
    BadVariableName(String),
}
