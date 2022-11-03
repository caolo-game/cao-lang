use std::str::FromStr;

use crate::{
    collections::{
        handle_table::{Handle, HandleTable},
        hash_map::CaoHashMap,
    },
    compiler::CardIndex,
    VarName,
};
use crate::{version, VariableId};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Labels(pub HandleTable<Label>);

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variables {
    pub ids: HandleTable<VariableId>,
    pub names: HandleTable<VarName>,
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
    pub trace: CaoHashMap<u32, CardIndex>,
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
