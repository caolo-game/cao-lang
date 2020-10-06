//! Helper module for dealing with function extensions.
//!
use crate::prelude::Scalar;
pub use crate::traits::Callable;
use crate::vm::VM;
use std::convert::TryFrom;
use std::marker::PhantomData;
use thiserror::Error;

pub type ExecutionResult = Result<(), ExecutionError>;

#[derive(Debug, Clone, Error)]
pub enum ExecutionError {
    #[error("Input ended unexpectedly")]
    UnexpectedEndOfInput,
    #[error("Program exited with status code: {0}")]
    ExitCode(i32),
    #[error("Got an invalid label: {0}")]
    InvalidLabel(i32),
    #[error("Got an invalid instruction code {0}")]
    InvalidInstruction(u8),
    #[error("Got an invalid argument to function call; {}",
        .context.as_ref().map(|x|x.as_str()).unwrap_or_else(|| ""))]
    InvalidArgument { context: Option<String> },
    #[error("Procedure by the name {0:?} could not be found")]
    ProcedureNotFound(String),
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
}

impl ExecutionError {
    pub fn invalid_argument<S: Into<Option<String>>>(reason: S) -> Self {
        Self::InvalidArgument {
            context: reason.into(),
        }
    }
}

pub struct Procedure<Aux> {
    fun: Box<dyn Callable<Aux>>,
}

impl<Aux> Callable<Aux> for Procedure<Aux> {
    fn call(&mut self, vm: &mut VM<Aux>, constants: &[Scalar]) -> ExecutionResult {
        self.fun.call(vm, constants)
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
    fn call(&mut self, vm: &mut VM<Aux>, _constants: &[Scalar]) -> ExecutionResult {
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
    fn call(&mut self, vm: &mut VM<Aux>, constants: &[Scalar]) -> ExecutionResult {
        let val = T::try_from(constants[0]).map_err(convert_error(0))?;
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
    fn call(&mut self, vm: &mut VM<Aux>, constants: &[Scalar]) -> ExecutionResult {
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
    F: Fn(&mut VM<Aux>, (T1, T2, T3)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, constants: &[Scalar]) -> ExecutionResult {
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
    return move |_| ExecutionError::invalid_argument(format!("Failed to convert argument {}", n));
}

macro_rules! callable_array_arg {
    ($num: expr) => {
        impl<Aux, F, T> Callable<Aux> for FunctionWrapper<Aux, F, [T; $num]>
        where
            F: Fn(&mut VM<Aux>, [T; $num]) -> ExecutionResult,
            T: TryFrom<Scalar>,
        {
            fn call(&mut self, vm: &mut VM<Aux>, constants: &[Scalar]) -> ExecutionResult {
                use arrayvec::ArrayVec;

                let args = constants.iter().enumerate().try_fold(ArrayVec::default(), |mut arr, (i, val)| {
                    let i = i as i32;
                    let val = T::try_from(*val).map_err(convert_error(i))?;
                    arr.push(val);
                    Ok(arr)
                })?;

                let args = args.into_inner().map_err(|_| {
                    ExecutionError::invalid_argument(None)
                })?;

                (self.f)(vm, args)
            }

            fn num_params(&self) -> u8 {
                $num
            }
        }
    };
    ($($nums: expr),*) => {
        $(callable_array_arg!($nums);)*
    };
}

callable_array_arg!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128
);
