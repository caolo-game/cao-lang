mod impls;

pub use self::impls::*;
use crate::{
    procedures::{ExecutionError, ExecutionResult},
    value::Value,
    vm::Vm,
};
use std::any::type_name;
use std::convert::TryFrom;

pub const MAX_STR_LEN: usize = 256;

// TODO: give better name? Do we even need it?
pub trait ByteEncodeble {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Debug)]
pub enum StringDecodeError {
    /// Could not decode lengt
    LengthDecodeError,
    /// Got an invalid length
    LengthError(usize),
    /// Did not fit into available space
    CapacityError(usize),
    Utf8DecodeError(std::str::Utf8Error),
}

/// Objects that can act as Cao-Lang functions
pub trait VmFunction<Aux> {
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult;
}

pub type VmFunction1<Aux, T1> = fn(&mut Vm<Aux>, T1) -> ExecutionResult;
pub type VmFunction2<Aux, T1, T2> = fn(&mut Vm<Aux>, T1, T2) -> ExecutionResult;
pub type VmFunction3<Aux, T1, T2, T3> = fn(&mut Vm<Aux>, T1, T2, T3) -> ExecutionResult;
pub type VmFunction4<Aux, T1, T2, T3, T4> = fn(&mut Vm<Aux>, T1, T2, T3, T4) -> ExecutionResult;

/// Casts the given function pointer to a Cao-Lang VM function taking 1 argument
///
/// See also:
///
/// - [into_f2]
/// - [into_f3]
/// - [into_f4]
///
/// ```
/// use cao_lang::prelude::*;
///
/// let mut vm = Vm::new(()).unwrap();
///
/// fn fun(_vm: &mut Vm<()>, _param: i64) -> ExecutionResult {
///     Ok(())
/// }
///
/// vm.register_function("my function", into_f1(fun));
///
/// ```
pub fn into_f1<Aux, T1>(f: fn(&mut Vm<Aux>, T1) -> ExecutionResult) -> VmFunction1<Aux, T1> {
    f as VmFunction1<_, _>
}

pub fn into_f2<Aux, T1, T2>(
    f: fn(&mut Vm<Aux>, T1, T2) -> ExecutionResult,
) -> VmFunction2<Aux, T1, T2> {
    f as VmFunction2<_, _, _>
}

pub fn into_f3<Aux, T1, T2, T3>(
    f: fn(&mut Vm<Aux>, T1, T2, T3) -> ExecutionResult,
) -> VmFunction3<Aux, T1, T2, T3> {
    f as VmFunction3<_, _, _, _>
}

pub fn into_f4<Aux, T1, T2, T3, T4>(
    f: fn(&mut Vm<Aux>, T1, T2, T3, T4) -> ExecutionResult,
) -> VmFunction4<Aux, T1, T2, T3, T4> {
    f as VmFunction4<_, _, _, _, _>
}

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
    T1: TryFrom<Value>,
{
    fn call(&self, vm: &mut Vm<Aux>) -> ExecutionResult {
        let v1 = vm.stack_pop();
        let v1 = T1::try_from(v1).map_err(|_| ExecutionError::invalid_argument(None))?;
        self(vm, v1)
    }
}

impl<Aux, T1, T2> VmFunction<Aux> for fn(&mut Vm<Aux>, T1, T2) -> ExecutionResult
where
    T1: TryFrom<Value>,
    T2: TryFrom<Value>,
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
    T1: TryFrom<Value>,
    T2: TryFrom<Value>,
    T3: TryFrom<Value>,
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
    T1: TryFrom<Value>,
    T2: TryFrom<Value>,
    T3: TryFrom<Value>,
    T4: TryFrom<Value>,
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
