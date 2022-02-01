#[cfg(feature = "serde")]
mod serde_impl;

use std::{collections::HashMap, str::FromStr};

use crate::{
    collections::key_map::{Handle, KeyMap},
    VarName,
};
use crate::{version, VariableId};

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Labels(pub KeyMap<Label>);

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variables {
    pub ids: KeyMap<VariableId>,
    /// maps the variableIds back to names for debugging purposes
    #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_impl::de_int_key"))]
    pub names: std::collections::HashMap<VariableId, VarName>,
}

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

/// Identifies a card in the program
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraceEntry {
    pub lane: i32,
    pub card: i32,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CaoProgram {
    /// Instructions
    pub bytecode: Vec<u8>,
    /// Data used by instuctions with variable length inputs
    pub data: Vec<u8>,
    pub labels: Labels,
    pub variables: Variables,
    pub cao_lang_version: (u8, u8, u16),
    pub trace: HashMap<usize, TraceEntry>,
}

impl CaoProgram {
    pub fn variable_id(&self, name: &str) -> Option<VariableId> {
        self.variables
            .ids
            .get(Handle::from_str(name).unwrap())
            .copied()
    }
}

impl Default for CaoProgram {
    fn default() -> Self {
        Self {
            bytecode: Default::default(),
            data: Default::default(),
            labels: Default::default(),
            variables: Default::default(),
            cao_lang_version: (version::MAJOR, version::MINOR, version::PATCH),
            trace: Default::default(),
        }
    }
}
