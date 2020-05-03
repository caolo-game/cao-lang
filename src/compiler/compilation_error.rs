use super::NodeId;
use crate::InputString;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum CompilationError {
    #[error("Program was empty")]
    EmptyProgram,
    #[error("No start node was found")]
    NoStart,

    #[error("SubProgram: [{0}] was not found")]
    MissingSubProgram(InputString),

    #[error("Program references node [{0}] but it was not found")]
    MissingNode(NodeId),
    /// Jumping from src to dst is illegal
    #[error("Jumping from {src} to {dst} can not be performed\n{msg:?}")]
    InvalidJump {
        src: NodeId,
        dst: NodeId,
        msg: Option<String>,
    },
}
