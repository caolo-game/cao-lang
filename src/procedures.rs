//! Helper module for dealing with function extensions.
//!
use crate::prelude::Scalar;
pub use crate::traits::Callable;
use crate::vm::VM;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::marker::PhantomData;

pub type ExecutionResult = Result<(), ExecutionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    UnexpectedEndOfInput,
    ExitCode(i32),
    InvalidLabel,
    InvalidInstruction,
    InvalidArgument,
    ProcedureNotFound(String),
    Unimplemented,
    OutOfMemory,
    MissingArgument,
    Timeout,
    TaskFailure(String),
}

pub struct Procedure<Aux> {
    fun: Box<dyn Callable<Aux>>,
}

impl<Aux> Callable<Aux> for Procedure<Aux> {
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        self.fun.call(vm, params)
    }

    fn num_params(&self) -> u8 {
        self.fun.num_params()
    }
}

impl<Aux> Procedure<Aux> {
    pub fn new<C: Callable<Aux> + 'static>(f: C) -> Self {
        Self { fun: Box::new(f) }
    }
}

impl<Aux> std::fmt::Debug for Procedure<Aux> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(writer, "Procedure")
    }
}

pub struct FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args) -> ExecutionResult,
{
    pub f: F,
    _args: PhantomData<(Args, Aux)>,
}

impl<Aux, F, Args> FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args) -> ExecutionResult,
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
    F: Fn(&mut VM<Aux>, ()) -> ExecutionResult,
{
    fn call(&mut self, vm: &mut VM<Aux>, _params: &[Scalar]) -> ExecutionResult {
        (self.f)(vm, ())
    }

    fn num_params(&self) -> u8 {
        0
    }
}

impl<Aux, F, T> Callable<Aux> for FunctionWrapper<Aux, F, T>
where
    F: Fn(&mut VM<Aux>, T) -> ExecutionResult,
    T: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let val = T::try_from(params[0]).map_err(convert_error(0))?;
        (self.f)(vm, val)
    }

    fn num_params(&self) -> u8 {
        1
    }
}

impl<Aux, F, T1, T2> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2)>
where
    F: Fn(&mut VM<Aux>, (T1, T2)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(params[0]).map_err(convert_error(0))?;
        let b = T2::try_from(params[1]).map_err(convert_error(1))?;
        (self.f)(vm, (a, b))
    }

    fn num_params(&self) -> u8 {
        2
    }
}

impl<Aux, F, T1, T2, T3> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2, T3)>
where
    F: Fn(&mut VM<Aux>, (T1, T2, T3)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(params[0]).map_err(convert_error(0))?;
        let b = T2::try_from(params[1]).map_err(convert_error(1))?;
        let c = T3::try_from(params[1]).map_err(convert_error(2))?;
        (self.f)(vm, (a, b, c))
    }

    fn num_params(&self) -> u8 {
        3
    }
}

fn convert_error<'a, T: 'a>(i: i32) -> impl Fn(T) -> ExecutionError + 'a {
    return move |_| {
        log::debug!("Failed to convert arugment #{}", i);
        ExecutionError::InvalidArgument
    };
}
