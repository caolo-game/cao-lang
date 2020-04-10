use crate::{procedures::ExecutionResult, scalar::Scalar, vm::VM};
use log::error;
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

pub trait ByteEncodeProperties: Sized + ObjectProperties {
    const BYTELEN: usize = mem::size_of::<Self>();
    type DecodeError;

    fn displayname() -> &'static str {
        type_name::<Self>()
    }
    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError>;
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

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;
    type DecodeError = StringDecodeError;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Vec<u8> {
        assert!(self.len() < Self::BYTELEN);
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.as_bytes());
        rr
    }

    fn decode(bytes: &[u8]) -> Result<Self, StringDecodeError> {
        let len = i32::decode(bytes).map_err(|_| {
            error!("Failed to deserialize length");
            StringDecodeError::LengthDecodeError
        })?;
        let len = usize::try_from(len).map_err(|e| {
            error!("Length must be non-negative, got: {}", e);
            StringDecodeError::LengthError(len)
        })?;
        std::str::from_utf8(&bytes[i32::BYTELEN..i32::BYTELEN + len])
            .map_err(|e| {
                error!("Failed to decode string {:?}", e);
                StringDecodeError::Utf8DecodeError(e)
            })
            .map(|s| s.to_owned())
    }
}

impl ByteEncodeProperties for () {
    const BYTELEN: usize = 0;
    type DecodeError = ();

    fn displayname() -> &'static str {
        "Void"
    }

    fn encode(self) -> Vec<u8> {
        vec![]
    }

    fn decode(_bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        Ok(())
    }
}

/// Opts in for the default implementation of ByteEncodeProperties
/// Note that using this with pointers, arrays, strings etc. will not work as one might expect!
pub trait AutoByteEncodeProperties {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

impl AutoByteEncodeProperties for i8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for f32 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}
impl AutoByteEncodeProperties for f64 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}

impl<T1: AutoByteEncodeProperties> AutoByteEncodeProperties for (T1,) {}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties> AutoByteEncodeProperties
    for (T1, T2)
{
}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties, T3: AutoByteEncodeProperties>
    AutoByteEncodeProperties for (T1, T2, T3)
{
}

impl<
        T1: AutoByteEncodeProperties,
        T2: AutoByteEncodeProperties,
        T3: AutoByteEncodeProperties,
        T4: AutoByteEncodeProperties,
    > AutoByteEncodeProperties for (T1, T2, T3, T4)
{
}

impl<T: std::fmt::Debug> ObjectProperties for T {}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeProperties
    for T
{
    type DecodeError = ();

    fn encode(self) -> Vec<u8> {
        let mut result = vec![0; Self::BYTELEN];
        unsafe {
            let dayum = mem::transmute::<*const Self, *const u8>(&self as *const Self);
            for i in 0..Self::BYTELEN {
                result[i] = *(dayum.add(i));
            }
        }
        result
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        if bytes.len() < Self::BYTELEN {
            Err(())
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Ok(result)
        }
    }
}

pub trait Callable<Aux> {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult;

    fn num_params(&self) -> u8;
}
