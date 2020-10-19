mod impls;

pub use self::impls::*;
use crate::{procedures::ExecutionResult, scalar::Scalar, vm::VM};
use std::any::type_name;
use std::convert::TryFrom;
use std::fmt::Write;
use std::mem;

pub const MAX_STR_LEN: usize = 256;

pub trait ObjectProperties: std::fmt::Debug {
    fn write_debug(&self, output: &mut String) {
        write!(output, "[object {:?}]", self).unwrap();
    }
}

pub trait ByteEncodeble: Sized + ObjectProperties {
    const BYTELEN: usize = mem::size_of::<Self>();

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

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError>;
}

pub trait DecodeInPlace<'a>: Sized + ObjectProperties + ByteEncodeble {
    type Ref;
    type DecodeError: std::fmt::Debug;

    fn decode_in_place(bytes: &'a [u8]) -> Result<Self::Ref, Self::DecodeError>;
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

/// Opts in for the default implementation of ByteEncodeProperties
/// Note that using this with pointers, arrays, strings etc. will not work as one might expect!
pub trait AutoByteEncodeProperties {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

pub trait Callable<Aux> {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult;

    fn num_params(&self) -> u8;
}