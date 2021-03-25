//! Helper module for dealing with function extensions.
//!
pub use crate::traits::Callable;
use crate::{collections::pre_hash_map::Key, prelude::Scalar};
use crate::{vm::Vm, InputString};
use std::convert::TryFrom;
use std::marker::PhantomData;
use thiserror::Error;

pub type ExecutionResult = Result<(), ExecutionError>;

#[derive(Debug, Clone, Error)]
pub enum ExecutionError {
    #[error("The program has overflown its call stack")]
    CallStackOverflow,
    #[error("Input ended unexpectedly")]
    UnexpectedEndOfInput,
    #[error("Program exited with status code: {0}")]
    ExitCode(i32),
    #[error("Got an invalid label hash: {0:?}")]
    InvalidLabel(Key),
    #[error("Got an invalid instruction code {0}")]
    InvalidInstruction(u8),
    #[error("Got an invalid argument to function call; {}",
        .context.as_ref().map(|x|x.as_str()).unwrap_or_else(|| ""))]
    InvalidArgument { context: Option<String> },
    #[error("Procedure by the hash {0:?} could not be found")]
    ProcedureNotFound(Key),
    #[error("Unimplemented")]
    Unimplemented,
    #[error("The program ran out of memory")]
    OutOfMemory,
    #[error("Missing argument to function call")]
    MissingArgument,
    #[error("Program timed out")]
    Timeout,
    #[error("Subtask failed {0:?}")]
    TaskFailure(String),
    #[error("The program has overflowns its stack")]
    Stackoverflow,
    #[error("Failed to return from a lane {reason}")]
    BadReturn { reason: String },
}

impl ExecutionError {
    pub fn invalid_argument<S: Into<Option<String>>>(reason: S) -> Self {
        Self::InvalidArgument {
            context: reason.into(),
        }
    }
}

pub(crate) struct Procedure<Aux> {
    pub name: InputString,
    pub fun: Box<dyn Callable<Aux>>,
}

impl<Aux> Callable<Aux> for Procedure<Aux> {
    fn call(&mut self, vm: &mut Vm<Aux>, constants: &[Scalar]) -> ExecutionResult {
        self.fun.call(vm, constants)
    }

    fn num_params(&self) -> u8 {
        self.fun.num_params()
    }
}

impl<Aux> Procedure<Aux> {
    pub fn new<C: Callable<Aux> + 'static>(name: InputString, f: C) -> Self {
        Self {
            fun: Box::new(f),
            name,
        }
    }
}

impl<Aux> std::fmt::Debug for Procedure<Aux> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(writer, "Procedure '{}'", self.name)
    }
}

pub struct FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut Vm<Aux>, Args) -> ExecutionResult,
{
    pub f: F,
    _args: PhantomData<(Args, Aux)>,
}

impl<Aux, F, Args> FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut Vm<Aux>, Args) -> ExecutionResult,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _args: Default::default(),
        }
    }
}

impl<Aux, F> Callable<Aux> for FunctionWrapper<Aux, F, ()>
where
    F: Fn(&mut Vm<Aux>, ()) -> ExecutionResult,
{
    fn call(&mut self, vm: &mut Vm<Aux>, _constants: &[Scalar]) -> ExecutionResult {
        (self.f)(vm, ())
    }

    fn num_params(&self) -> u8 {
        0
    }
}

impl<Aux, F, T> Callable<Aux> for FunctionWrapper<Aux, F, T>
where
    F: Fn(&mut Vm<Aux>, T) -> ExecutionResult,
    T: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut Vm<Aux>, constants: &[Scalar]) -> ExecutionResult {
        let val = T::try_from(constants[0]).map_err(convert_error(0))?;
        (self.f)(vm, val)
    }

    fn num_params(&self) -> u8 {
        1
    }
}

impl<Aux, F, T1, T2> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2)>
where
    F: Fn(&mut Vm<Aux>, (T1, T2)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut Vm<Aux>, constants: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(constants[0]).map_err(convert_error(0))?;
        let b = T2::try_from(constants[1]).map_err(convert_error(1))?;
        (self.f)(vm, (a, b))
    }

    fn num_params(&self) -> u8 {
        2
    }
}

impl<Aux, F, T1, T2, T3> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2, T3)>
where
    F: Fn(&mut Vm<Aux>, (T1, T2, T3)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut Vm<Aux>, constants: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(constants[0]).map_err(convert_error(0))?;
        let b = T2::try_from(constants[1]).map_err(convert_error(1))?;
        let c = T3::try_from(constants[1]).map_err(convert_error(2))?;
        (self.f)(vm, (a, b, c))
    }

    fn num_params(&self) -> u8 {
        3
    }
}

fn convert_error<'a, T: 'a>(n: i32) -> impl Fn(T) -> ExecutionError + 'a {
    move |_| ExecutionError::invalid_argument(format!("Failed to convert argument {}", n))
}

impl<Aux, F, T, const LEN: usize> Callable<Aux> for FunctionWrapper<Aux, F, [T; LEN]>
where
    F: Fn(&mut Vm<Aux>, [T; LEN]) -> ExecutionResult,
    T: TryFrom<Scalar>,
    [T; LEN]: arrayvec::Array<Item = T>,
{
    fn call(&mut self, vm: &mut Vm<Aux>, constants: &[Scalar]) -> ExecutionResult {
        use arrayvec::ArrayVec;

        let args =
            constants
                .iter()
                .enumerate()
                .try_fold(ArrayVec::default(), |mut arr, (i, val)| {
                    let i = i as i32;
                    let val = T::try_from(*val).map_err(convert_error(i))?;
                    arr.push(val);
                    Ok(arr)
                })?;

        let args = args
            .into_inner()
            .map_err(|_| ExecutionError::invalid_argument(None))?;

        (self.f)(vm, args)
    }

    fn num_params(&self) -> u8 {
        debug_assert!((LEN & std::u8::MAX as usize) == LEN);
        LEN as u8
    }
}
