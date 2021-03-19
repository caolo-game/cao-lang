use std::str::FromStr;

use crate::collections::pre_hash_map::{Key, PreHashMap};
use crate::{version, VariableId};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Labels(pub PreHashMap<Label>);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variables(pub PreHashMap<VariableId>);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    /// Position of this card in the bytecode of the program
    pub pos: u32,
}

impl Label {
    pub fn new(pos: u32) -> Self {
        Self { pos }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompiledProgram {
    /// Bytecode layout: (instr [data])*
    pub bytecode: Vec<u8>,
    pub labels: Labels,
    pub variables: Variables,
    pub cao_lang_version: (u8, u8, u16),
}

impl CompiledProgram {
    pub fn variable_id(&self, name: &str) -> Option<VariableId> {
        self.variables
            .0
            .get(Key::from_str(name).unwrap())
            .map(|x| *x) // trigger an error if VariableId is no longer Copy...
    }
}

impl Default for CompiledProgram {
    fn default() -> Self {
        CompiledProgram {
            bytecode: Default::default(),
            labels: Default::default(),
            variables: Default::default(),
            cao_lang_version: (version::MAJOR, version::MINOR, version::PATCH),
        }
    }
}
