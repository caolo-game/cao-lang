mod impls;

pub use self::impls::*;
use crate::{procedures::ExecutionResult, scalar::Scalar, vm::Vm};
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
pub trait Callable<Aux> {
    /// Take in the Vm, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(&mut self, vm: &mut Vm<Aux>, params: &[Scalar]) -> ExecutionResult;
    fn num_params(&self) -> u8;
}
