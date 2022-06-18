//! ## Programs
//!
//! __TBA__
//!

#![recursion_limit = "256"]

mod alloc;
pub mod collections;
pub mod compiled_program;
pub mod compiler;
pub mod instruction;
pub mod prelude;
pub mod procedures;
pub mod traits;
pub mod value;
pub mod vm;

mod bytecode;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_lang_version.rs"));
}

use std::{cmp::Ordering, mem::size_of, str::FromStr};

use crate::instruction::Instruction;
use arrayvec::ArrayString;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableId(u32);

impl FromStr for VariableId {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = u32::from_str(s)?;
        Ok(VariableId(inner))
    }
}

/// Unique id of each nodes in a single compilation
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId {
    /// Index of the lane this node is in
    pub lane: u32,
    /// Index of the node relative to the lane
    pub pos: u32,
}

impl From<NodeId> for u64 {
    fn from(n: NodeId) -> u64 {
        ((n.lane as u64) << 32) | n.pos as u64
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct StrPointer(pub *mut u8);

impl StrPointer {
    /// # Safety
    ///
    /// Must be called with ptr obtained from a `string_literal` instruction, before the last `clear`!
    ///
    /// # Return value
    ///
    /// Returns None if the underlying string is not valid utf8
    pub unsafe fn get_str<'a>(self) -> Option<&'a str> {
        let ptr = self.0;
        let len = *(ptr as *const u32);
        let ptr = ptr.add(size_of::<u32>());
        std::str::from_utf8(std::slice::from_raw_parts(ptr, len as usize)).ok()
    }
}

pub(crate) const INPUT_STR_LEN_IN_BYTES: usize = 255;

pub type InputString = ArrayString<INPUT_STR_LEN_IN_BYTES>;
pub type VarName = ArrayString<64>;

/// Metadata about a subprogram in the program.
/// Subprograms consume their inputs and produce outputs.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubProgramDescription {
    pub name: String,
    pub description: String,
    pub ty: SubProgramType,

    /// Human readable descriptions of the output
    pub output: Box<[String]>,
    /// Human readable descriptions of inputs
    pub input: Box<[String]>,
    /// Human readable descriptions of properties
    pub properties: Box<[String]>,
}

impl std::fmt::Debug for SubProgramDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubProgram")
            .field("name", &self.name)
            .field("input", &self.input)
            .field("output", &self.output)
            .field("properties", &self.properties)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubProgramType {
    Undefined,
    /// Most basic nodes, translated to (virtual) machine instructions
    Instruction,
    /// Some black box
    Function,
    /// Branching nodes may redirect the flow of the program
    Branch,
    /// Cards related to objects
    Object,
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
            SubProgramType::Object => "Object",
        }
    }
}

#[macro_export]
macro_rules! subprogram_description {
    ($name: expr, $description: expr, $ty: expr, [$($inputs: expr),*], [$($outputs: expr),*], [$($properties: expr),*]) => {
        SubProgramDescription {
            name: $name.to_string(),
            description: $description.to_string(),
            ty: $ty,
            input: subprogram_description!(@input $($inputs),*) ,
            output: subprogram_description!(@input $($outputs),*),
            properties: subprogram_description!(@input $($properties),*),
        }
    };

    (@input $($lst: expr),*) => {
        vec![
            $(
                $lst.to_string()
            ),*
        ]
        .into_boxed_slice()
    };

    (@input) => {
        Box::new()
    };
}
