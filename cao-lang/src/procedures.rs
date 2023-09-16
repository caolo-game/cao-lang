//! Helper module for dealing with function extensions.
//!
use std::fmt::Display;
use std::ptr::NonNull;

use crate::collections::handle_table::Handle;
use crate::prelude::Trace;
use crate::traits::VmFunction;
use crate::vm::runtime::cao_lang_object::CaoLangObject;
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
    #[error("Got an invalid argument: {}",
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
    #[error("Assertion failed: {0}")]
    AssertionError(String),
    #[error("Closure requested a non-existent upvalue")]
    InvalidUpvalue,
    #[error("Expected to be in the context of a closure")]
    NotClosure,
}

#[derive(Debug, Clone, Error)]
pub struct ExecutionError {
    pub payload: ExecutionErrorPayload,
    pub trace: Vec<Trace>,
}

impl ExecutionError {
    pub fn new(payload: ExecutionErrorPayload, trace: Vec<Trace>) -> Self {
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
        S: Into<String>,
    {
        Self::InvalidArgument {
            context: Some(reason.into()),
        }
    }
}

pub(crate) struct Procedure<Aux> {
    pub fun: std::rc::Rc<dyn VmFunction<Aux>>,
    pub name: NonNull<CaoLangObject>,
}

impl<Aux> Clone for Procedure<Aux> {
    fn clone(&self) -> Self {
        Self {
            fun: self.fun.clone(),
            name: self.name,
        }
    }
}

impl<Aux> Procedure<Aux> {
    pub fn name(&self) -> &str {
        unsafe { self.name.as_ref().as_str().unwrap() }
    }
}

impl<Aux> std::fmt::Debug for Procedure<Aux> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            writeln!(
                writer,
                "Procedure '{}'",
                self.name.as_ref().as_str().unwrap()
            )
        }
    }
}
