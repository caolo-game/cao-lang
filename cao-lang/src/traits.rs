mod impls;

pub use self::impls::*;
use crate::{
    procedures::{ExecutionError, ExecutionResult},
    scalar::Scalar,
    vm::Vm,
};
use std::any::type_name;
use std::convert::TryFrom;
use std::fmt::Write;

pub const MAX_STR_LEN: usize = 256;

pub trait ObjectProperties: std::fmt::Debug {
    fn write_debug(&self, output: &mut String) {
        write!(output, "[object {:?}]", self).unwrap();
    }
}

pub trait ByteEncodeble: Sized + ObjectProperties {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

pub trait ByteEncodeProperties: Sized + ObjectProperties + ByteEncodeble {
    type EncodeError: std::fmt::Debug;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError>;
}

pub trait ByteDecodeProperties: Sized + ObjectProperties + ByteEncodeble {
    type DecodeError: std::fmt::Debug;

    /// return the bytes read
    fn decode(bytes: &[u8]) -> Result<(usize, Self), Self::DecodeError>;

    /// return the bytes read
    ///
    /// # Safety
    ///
    /// Can assume that the underlying data represents this type
    unsafe fn decode_unsafe(bytes: &[u8]) -> (usize, Self);
}

pub trait DecodeInPlace<'a>: Sized + ObjectProperties + ByteEncodeble {
    type Ref;
    type DecodeError: std::fmt::Debug;

    /// return the bytes read
    fn decode_in_place(bytes: &'a [u8]) -> Result<(usize, Self::Ref), Self::DecodeError>;
}

#[derive(Debug)]
pub enum StringDecodeError {
    /// Could not decode lengt
    LengthDecodeError,
    /// Got an invalid length
    LengthError(i32),
    /// Did not fit into available space
    CapacityError(usize),
    Utf8DecodeError(std::str::Utf8Error),
}

/// Opts in for the default implementation of ByteEncodeProperties which is memcopy
pub trait AutoByteEncodeProperties: Copy + std::fmt::Debug {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

/// Objects that can act as Cao-Lang functions
pub trait VmFunction<Aux> {
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult;
}

/// Type alias for free functions with 1 parameter
///
/// ```
/// use cao_lang::prelude::*;
///
/// let mut vm = Vm::new(());
///
/// fn fun(_vm: &mut Vm<()>, _param: i32) -> ExecutionResult {
///     Ok(())
/// }
///
/// // first type argument is the auxiliary data, the second the the parameter type
/// vm.register_function("my function", fun as VmFunction1<_, _>);
/// ```
pub type VmFunction1<Aux, T1> = fn(&mut Vm<Aux>, T1) -> ExecutionResult;
pub type VmFunction2<Aux, T1, T2> = fn(&mut Vm<Aux>, T1, T2) -> ExecutionResult;
pub type VmFunction3<Aux, T1, T2, T3> = fn(&mut Vm<Aux>, T1, T2, T3) -> ExecutionResult;
pub type VmFunction4<Aux, T1, T2, T3, T4> = fn(&mut Vm<Aux>, T1, T2, T3, T4) -> ExecutionResult;

impl<Aux, F> VmFunction<Aux> for F
where
    F: Fn(&mut Vm<Aux>) -> ExecutionResult,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        self(vm)
    }
}

impl<Aux, T1> VmFunction<Aux> for VmFunction1<Aux, T1>
where
    T1: TryFrom<Scalar>,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        let v1 = vm.stack_pop();
        let v1 = T1::try_from(v1).map_err(|_| ExecutionError::invalid_argument(None))?;
        self(vm, v1)
    }
}

impl<Aux, T1, T2> VmFunction<Aux> for fn(&mut Vm<Aux>, T1, T2) -> ExecutionResult
where
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        let v2 = vm.stack_pop();
        let v2 = T2::try_from(v2).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v1 = vm.stack_pop();
        let v1 = T1::try_from(v1).map_err(|_| ExecutionError::invalid_argument(None))?;
        self(vm, v1, v2)
    }
}

impl<Aux, T1, T2, T3> VmFunction<Aux> for fn(&mut Vm<Aux>, T1, T2, T3) -> ExecutionResult
where
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        let v3 = vm.stack_pop();
        let v3 = T3::try_from(v3).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v2 = vm.stack_pop();
        let v2 = T2::try_from(v2).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v1 = vm.stack_pop();
        let v1 = T1::try_from(v1).map_err(|_| ExecutionError::invalid_argument(None))?;
        self(vm, v1, v2, v3)
    }
}

impl<Aux, T1, T2, T3, T4> VmFunction<Aux> for fn(&mut Vm<Aux>, T1, T2, T3, T4) -> ExecutionResult
where
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
    T4: TryFrom<Scalar>,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        let v4 = vm.stack_pop();
        let v4 = T4::try_from(v4).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v3 = vm.stack_pop();
        let v3 = T3::try_from(v3).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v2 = vm.stack_pop();
        let v2 = T2::try_from(v2).map_err(|_| ExecutionError::invalid_argument(None))?;
        let v1 = vm.stack_pop();
        let v1 = T1::try_from(v1).map_err(|_| ExecutionError::invalid_argument(None))?;
        self(vm, v1, v2, v3, v4)
    }
}
