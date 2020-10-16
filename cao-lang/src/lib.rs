//! ## Programs
//!
//! Programs are composed of subprograms. A subprogram consumes inputs and produces outputs.
//! Subprograms will always consume from the top of the stack downwards and push their outputs to
//! the stack. This means that subprogram composition is not a commutative operation. (Consider
//! subprograms A, B and C. Then the composition ABC is not the same as BAC if A != B. )
//!
//! Programs passed to the `Compiler` must contain a `Start` node. Execution will begin at the
//! first `Start` node.
//!
//! Example (Sub) Program serialized as JSON
//! ```
//! const PROGRAM: &str = r#"{
//!     "nodes": {
//!         "0": {
//!             "node": {
//!                 "Start": null
//!             },
//!             "child": 1
//!         },
//!         "1": {
//!             "node": {
//!                 "ScalarInt": 42
//!             },
//!             "child": 2
//!         },
//!         "2": {
//!             "node": {
//!                 "Call": "log_scalar"
//!             }
//!         }
//!     }
//! }"#;
//!
//! let compilation_unit = serde_json::from_str(PROGRAM).unwrap();
//! cao_lang::compiler::compile(None, compilation_unit).unwrap();
//!```
//!

#![recursion_limit = "256"]

pub mod compiler;
pub mod instruction;
mod macros;
pub mod prelude;
pub mod procedures;
pub mod scalar;
pub mod traits;
pub mod vm;

use crate::instruction::Instruction;
use crate::{compiler::NodeId, traits::AutoByteEncodeProperties};
use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub struct Pointer(pub i32);

impl Into<i32> for Pointer {
    fn into(self) -> i32 {
        self.0
    }
}

impl AsRef<i32> for Pointer {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl AsMut<i32> for Pointer {
    fn as_mut(&mut self) -> &mut i32 {
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

pub type Labels = HashMap<NodeId, Label>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompiledProgram {
    /// Bytecode layout: (instr node_id [data])+
    pub bytecode: Vec<u8>,
    /// Label: [block, self]
    pub labels: Labels,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Label {
    /// Index of the beginning in the bytecode of the program
    pub block: u32,
}

impl Label {
    pub fn new(block: u32) -> Self {
        Self { block }
    }
}
pub type VarName = ArrayString<[u8; 64]>;
impl AutoByteEncodeProperties for VarName {}

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
