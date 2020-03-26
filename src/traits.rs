use crate::{scalar::Scalar, vm::VM, procedures::ExecutionResult};
use std::any::type_name;
use std::fmt::Write;
use std::mem;

pub const MAX_STR_LEN: usize = 128;

pub trait ObjectProperties: std::fmt::Debug {
    fn write_debug(&self, output: &mut String) {
        write!(output, "[object {:?}]", self).unwrap();
    }
}

pub trait ByteEncodeProperties: Sized + ObjectProperties {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn displayname() -> &'static str {
        type_name::<Self>()
    }
    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Vec<u8> {
        assert!(self.len() < Self::BYTELEN);
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.chars().map(|c| c as u8));
        rr
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let len = i32::decode(bytes)?;
        let string = bytes
            .iter()
            .skip(i32::BYTELEN)
            .take(len as usize)
            .map(|c| *c as char)
            .collect();
        Some(string)
    }
}

impl ByteEncodeProperties for () {
    const BYTELEN: usize = 0;
    fn displayname() -> &'static str {
        "Void"
    }

    fn encode(self) -> Vec<u8> {
        vec![]
    }

    fn decode(_bytes: &[u8]) -> Option<Self> {
        None
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
    fn encode(self) -> Vec<u8> {
        let size: usize = Self::BYTELEN;

        let mut result = vec![0; size];
        unsafe {
            let dayum = mem::transmute::<*const Self, *const u8>(&self as *const Self);
            for i in 0..size {
                result[i] = *(dayum.add(i));
            }
        }
        result
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let size: usize = Self::BYTELEN;
        if bytes.len() < size {
            None
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Some(result)
        }
    }
}

pub trait Callable<Aux> {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult;

    fn num_params(&self) -> u8;
}
