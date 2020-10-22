use crate::collections::pre_hash_map::PreHashMap;
use crate::{version, VariableId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Labels(pub PreHashMap<Label>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Variables(pub PreHashMap<VariableId>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Label {
    /// Position of this card in the bytecode of the program
    pub pos: u32,
}

impl Label {
    pub fn new(pos: u32) -> Self {
        Self { pos }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProgram {
    /// Bytecode layout: (instr [data])*
    pub bytecode: Vec<u8>,
    pub labels: Labels,
    pub variables: Variables,
    pub cao_lang_version: (u8, u8, u16),
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
