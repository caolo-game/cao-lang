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
//!             "children": [
//!                 1
//!             ]
//!         },
//!         "1": {
//!             "node": {
//!                 "ScalarInt": {
//!                     "value": 42
//!                 }
//!             },
//!             "children": [
//!                 2
//!             ]
//!         },
//!         "2": {
//!             "node": {
//!                 "Call": {
//!                     "function": "log_scalar"
//!                 }
//!             }
//!         }
//!     }
//! }"#;
//!
//! let compilation_unit = serde_json::from_str(PROGRAM).unwrap();
//! cao_lang::compiler::Compiler::compile(compilation_unit).unwrap();
//!```
//!
pub mod compiler;
pub mod instruction;
mod macros;
pub mod prelude;
pub mod procedures;
pub mod scalar;
pub mod traits;
pub mod vm;

use crate::compiler::NodeId;
use crate::instruction::Instruction;
use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;

pub type TPointer = i32;

pub const MAX_INPUT_PER_NODE: usize = 8;
pub const INPUT_STR_LEN: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN]>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompiledProgram {
    pub bytecode: Vec<u8>,
    /// Label: [block, self]
    pub labels: HashMap<NodeId, Label>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Label {
    pub block: u32,
    pub myself: u32,
}

impl Label {
    pub fn new(block: u32, myself: u32) -> Self {
        Self { block, myself }
    }
}

impl Index<i32> for Label {
    type Output = u32;
    fn index(&self, ind: i32) -> &Self::Output {
        match ind {
            0 => &self.block,
            1 => &self.myself,
            _ => unreachable!("Label index must be 0 or 1"),
        }
    }
}

pub type VarName = ArrayString<[u8; 64]>;
impl crate::traits::AutoByteEncodeProperties for VarName {}

/// Metadata about a subprogram in the program.
/// Subprograms consume their inputs and produce outputs.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SubProgram<'a> {
    pub name: &'a str,
    pub description: &'a str,
    /// Human readable descriptions of the output
    pub output: Vec<&'a str>,
    /// Human readable descriptions of inputs
    pub input: Vec<&'a str>,
}

impl<'a> std::fmt::Debug for SubProgram<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function name: {} inputs: {} outputs: {}",
            self.name,
            self.input[..].join(", "),
            self.output[..].join(", ")
        )
    }
}

#[macro_export]
macro_rules! subprogram_description {
    ($name: ident, $description: expr, [$($inputs: ty),*], [$($outputs: ty),*]) => {
        SubProgram {
            name: stringify!($name),
            description: $description,
            input: subprogram_description!(input $($inputs),*) ,
            output: subprogram_description!(input $($outputs),*),
        }
    };

    (input $($lst:ty),*) => {
        vec![
            $(
                <$lst as ByteEncodeProperties>::displayname()
            ),*
        ]
    };

    (input) => {
        vec![]
    };
}
