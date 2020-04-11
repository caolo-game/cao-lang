use super::NodeId;
use serde_derive::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use crate::InputString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilationError {
    EmptyProgram,
    NoStart,
    MissingSubProgram(InputString),
    /// Node was referenced but not found
    MissingNode(NodeId),
    /// Jumping from src to dst is illegal
    InvalidJump {
        src: NodeId,
        dst: NodeId,
        msg: Option<String>,
    },
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::EmptyProgram => write!(f, "Program was empty"),
            CompilationError::NoStart => write!(f, "Program had no start node"),
            CompilationError::MissingNode(nodeid) => write!(
                f,
                "Program references node [{}] but it was not found",
                nodeid
            ),
            CompilationError::InvalidJump { src, dst, msg } => {
                if let Some(msg) = msg.as_ref() {
                    write!(f, "{}", msg)
                } else {
                    write!(f, "Jumping from {} to {} can not be performed", src, dst)
                }
            }
            CompilationError::MissingSubProgram(b) => write!(f, "SubProgram: [{}] was not found", b),
        }
    }
}
