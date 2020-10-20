//! ## Programs
//!
//! Programs are composed of Lanes and Cards. A Card consumes inputs and produces outputs.
//! Cards will always consume from the top of the stack downwards and push their outputs to
//! the stack. This means that subprogram composition is not a commutative operation. (Consider
//! Cards A, B and C. Then the composition ABC is not the same as BAC if A != B. )
//!
//! Execution will begin at the first `Lane`.
//!
//! Example Program serialized as JSON
//! ```
//! const PROGRAM: &str = r#"{
//!   "lanes":[
//!      {
//!         "name":"Main",
//!         "cards":[
//!            {
//!               "ScalarInt":42
//!            },
//!            {
//!               "Call":"log_scalar"
//!            }
//!         ]
//!      }
//!   ]
//!}"#;
//!
//!let compilation_unit = serde_json::from_str(PROGRAM).unwrap();
//!cao_lang::compiler::compile(None, compilation_unit, None).unwrap();
//!```
//!

#![recursion_limit = "256"]

pub mod compiler;
pub mod instruction;
mod macros;
pub mod prelude;
pub mod procedures;
pub mod program;
pub mod scalar;
pub mod traits;
pub mod vm;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_lang_version.rs"));
}

use std::cmp::Ordering;

use crate::instruction::Instruction;
use crate::traits::AutoByteEncodeProperties;
use arrayvec::ArrayString;
use prelude::{ByteDecodeProperties, ByteEncodeProperties, ByteEncodeble, StringDecodeError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct VariableId(u32);
impl AutoByteEncodeProperties for VariableId {}

/// Unique id of each nodes in a single compilation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct NodeId {
    /// Index of the lane this node is in
    pub lane: u16,
    /// Index of the node relative to the lane
    pub pos: u16,
}

impl Into<u32> for NodeId {
    fn into(self) -> u32 {
        ((self.lane as u32) << 16) | self.pos as u32
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.lane
            .cmp(&other.lane)
            .then_with(move || self.pos.cmp(&other.pos))
    }
}

impl AutoByteEncodeProperties for NodeId {
    fn displayname() -> &'static str {
        "Card ID"
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub struct Pointer(pub u32);

impl Into<u32> for Pointer {
    fn into(self) -> u32 {
        self.0
    }
}

impl AsRef<u32> for Pointer {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl AsMut<u32> for Pointer {
    fn as_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}

impl AutoByteEncodeProperties for Pointer {
    fn displayname() -> &'static str {
        "Object Reference"
    }
}

pub const MAX_INPUT_PER_NODE: usize = 8;
pub const INPUT_STR_LEN_IN_BYTES: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN_IN_BYTES]>;

pub type VarName = ArrayString<[u8; 64]>;

impl ByteEncodeble for VarName {
    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeProperties for VarName {
    type EncodeError = StringDecodeError;

    fn encode(self, out: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        (self.len() as i32)
            .encode(out)
            .expect("failed to encode i32");
        out.extend(self.as_bytes());
        Ok(())
    }
}
impl ByteDecodeProperties for VarName {
    type DecodeError = StringDecodeError;

    fn decode(bytes: &[u8]) -> Result<(usize, Self), Self::DecodeError> {
        use std::convert::TryFrom;

        let (ll, len) = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        if len > 64 {
            return Err(StringDecodeError::LengthError(len as i32));
        }
        let val = std::str::from_utf8(&bytes[ll..ll + len])
            .map_err(StringDecodeError::Utf8DecodeError)?;
        let val = ArrayString::from(val).expect("failed to convert str to array");
        Ok((len + ll, val))
    }
}

impl<'a> ByteEncodeble for &'a str {
    fn displayname() -> &'static str {
        "Text"
    }
}

/// Metadata about a subprogram in the program.
/// Subprograms consume their inputs and produce outputs.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SubProgram<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub ty: SubProgramType,

    /// Human readable descriptions of the output
    pub output: Box<[&'a str]>,
    /// Human readable descriptions of inputs
    pub input: Box<[&'a str]>,
    /// Human readable descriptions of parameters
    pub constants: Box<[&'a str]>,
}

impl<'a> std::fmt::Debug for SubProgram<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubProgram")
            .field("name", &self.name)
            .field("input", &self.input)
            .field("output", &self.output)
            .field("constants", &self.constants)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SubProgramType {
    /// Any ol' sub-program
    Undefined,
    /// Most basic nodes, translated to (virtual) machine instructions
    Instruction,
    /// Some black box
    Function,
    /// Branching nodes may redirect the flow of the program
    Branch,
    /// Programs start with a start node
    Start,
}

impl Default for SubProgramType {
    fn default() -> Self {
        SubProgramType::Undefined
    }
}

#[macro_export]
macro_rules! subprogram_description {
    ($name: expr, $description: expr, $ty: expr, [$($inputs: ty),*], [$($outputs: ty),*], [$($constants: ty),*]) => {
        SubProgram {
            name: $name,
            description: $description,
            ty: $ty,
            input: subprogram_description!(@input $($inputs),*) ,
            output: subprogram_description!(@input $($outputs),*),
            constants: subprogram_description!(@input $($constants),*),
        }
    };

    (@input $($lst:ty),*) => {
        vec![
            $(
                <$lst as ByteEncodeble>::displayname()
            ),*
        ]
        .into_boxed_slice()
    };

    (@input) => {
        Box::new()
    };
}
