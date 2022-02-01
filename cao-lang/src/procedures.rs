//! Helper module for dealing with function extensions.
//!
use std::fmt::Display;

use crate::collections::key_map::Handle;
use crate::prelude::TraceEntry;
use crate::traits::VmFunction;
use thiserror::Error;

pub type ExecutionResult<T = ()> = Result<T, ExecutionError>;

#[derive(Debug, Clone, Error)]
pub enum ExecutionErrorPayload {
    #[error("The program has overflown its call stack")]
    CallStackOverflow,
    #[error("Input ended unexpectedly")]
    UnexpectedEndOfInput,
    #[error("Program exited with status code: {0}")]
    ExitCode(i32),
    #[error("Got an invalid instruction code {0}")]
    InvalidInstruction(u8),
    #[error("Got an invalid argument to function call; {}",
        .context.as_ref().map(|x|x.as_str()).unwrap_or_else(|| ""))]
    InvalidArgument { context: Option<String> },
    #[error("Variable {0} was not found!")]
    VarNotFound(String),
    #[error("Procedure by the hash {0:?} could not be found")]
    ProcedureNotFound(Handle),
    #[error("Unimplemented")]
    Unimplemented,
    #[error("The program ran out of memory")]
    OutOfMemory,
    #[error("Missing argument to function call")]
    MissingArgument,
    #[error("Program timed out")]
    Timeout,
    #[error("Subtask [{name}] failed {error}")]
    TaskFailure {
        name: String,
        error: Box<ExecutionErrorPayload>,
    },
    #[error("The program has overflowns its stack")]
    Stackoverflow,
    #[error("Failed to return from a lane {reason}")]
    BadReturn { reason: String },
    #[error("Trying to hash an unhashable object")]
    Unhashable,
}

#[derive(Debug, Clone, Error)]
pub struct ExecutionError {
    pub payload: ExecutionErrorPayload,
    pub trace: TraceEntry,
}

impl ExecutionError {
    pub fn new(payload: ExecutionErrorPayload, trace: TraceEntry) -> Self {
        Self { payload, trace }
    }
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecutionError: {}", self.payload)
    }
}

impl ExecutionErrorPayload {
    pub fn invalid_argument<S>(reason: S) -> Self
    where
        S: Into<Option<String>>,
    {
        Self::InvalidArgument {
            context: reason.into(),
        }
    }
}

pub(crate) struct Procedure<Aux> {
    pub fun: Box<dyn VmFunction<Aux>>,
    pub name: String,
}

impl<Aux> Procedure<Aux> {
    pub fn new<S: Into<String>, C: VmFunction<Aux> + 'static>(name: S, f: C) -> Self {
        Self {
            fun: Box::new(f),
            name: name.into(),
        }
    }
}

impl<Aux> std::fmt::Debug for Procedure<Aux> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(writer, "Procedure '{}'", self.name)
    }
}
