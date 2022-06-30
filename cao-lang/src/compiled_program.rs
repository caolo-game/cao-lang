use std::{collections::HashMap, str::FromStr};

use crate::{
    collections::key_map::{Handle, KeyMap},
    VarName,
};
use crate::{version, VariableId};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Labels(pub KeyMap<Label>);

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variables {
    pub ids: KeyMap<VariableId>,
    pub names: KeyMap<VarName>,
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
    pub lane: Box<str>,
    pub card: i32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CaoCompiledProgram {
    /// Instructions
    pub bytecode: Vec<u8>,
    /// Data used by instuctions with variable length inputs
    pub data: Vec<u8>,
    pub labels: Labels,
    pub variables: Variables,
    pub cao_lang_version: (u8, u8, u16),
    pub trace: HashMap<usize, TraceEntry>,
}

impl CaoCompiledProgram {
    pub fn variable_id(&self, name: &str) -> Option<VariableId> {
        self.variables
            .ids
            .get(Handle::from_str(name).unwrap())
            .copied()
    }
}

impl Default for CaoCompiledProgram {
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
