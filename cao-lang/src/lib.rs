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
//!               "ty": "ScalarInt",
//!               "val":42
//!            },
//!            {
//!               "ty": "CallNative", "val":"log_scalar"
//!            }
//!         ]
//!      }
//!   ]
//!}"#;
//!
//!let compilation_unit = serde_json::from_str(PROGRAM).unwrap();
//!cao_lang::compiler::compile(compilation_unit, None).unwrap();
//!```
//!

#![recursion_limit = "256"]

mod alloc;
pub mod collections;
pub mod compiler;
pub mod instruction;
pub mod prelude;
pub mod procedures;
pub mod program;
pub mod value;
pub mod traits;
pub mod vm;

mod macros;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_lang_version.rs"));
}

use std::cmp::Ordering;

use crate::instruction::Instruction;
use crate::traits::AutoByteEncodeProperties;
use arrayvec::ArrayString;
use prelude::{ByteEncodeProperties, ByteEncodeble, StringDecodeError};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableId(u32);
impl AutoByteEncodeProperties for VariableId {}

/// Unique id of each nodes in a single compilation
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId {
    /// Index of the lane this node is in
    pub lane: u16,
    /// Index of the node relative to the lane
    pub pos: u16,
}

impl From<NodeId> for u32 {
    fn from(n: NodeId) -> u32 {
        ((n.lane as u32) << 16) | n.pos as u32
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
        "Node ID"
    }
}

/// Memory handles
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pointer(pub *mut [u8]);

impl AutoByteEncodeProperties for Pointer {
    fn displayname() -> &'static str {
        "Object Reference"
    }
}

pub(crate) const INPUT_STR_LEN_IN_BYTES: usize = 255;

pub type InputString = ArrayString<INPUT_STR_LEN_IN_BYTES>;
pub type VarName = ArrayString<64>;

impl ByteEncodeble for VarName {
    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeProperties for VarName {
    type EncodeError = StringDecodeError;

    fn layout(&self) -> std::alloc::Layout {
        <&str as ByteEncodeProperties>::layout(&self.as_str())
    }

    fn encode(self, out: &mut [u8]) -> Result<(), Self::EncodeError> {
        <&str as ByteEncodeProperties>::encode(self.as_str(), out)
    }
}

/// Metadata about a subprogram in the program.
/// Subprograms consume their inputs and produce outputs.
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl SubProgramType {
    pub fn as_str(self) -> &'static str {
        match self {
            SubProgramType::Undefined => "Undefined",
            SubProgramType::Instruction => "Instruction",
            SubProgramType::Function => "Function",
            SubProgramType::Branch => "Branch",
            SubProgramType::Start => "Start",
        }
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
