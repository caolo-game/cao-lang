use crate::{NodeId, VarName, VariableId};
use serde::{de::SeqAccess, de::Visitor, ser::SerializeSeq, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Labels(pub HashMap<NodeId, Label>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Variables(pub HashMap<VarName, VariableId>);

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
    pub cao_lang_version: String,
}

impl Default for CompiledProgram {
    fn default() -> Self {
        CompiledProgram {
            bytecode: Default::default(),
            labels: Default::default(),
            variables: Default::default(),
            cao_lang_version: crate::version::VERSION_STR.to_owned(),
        }
    }
}

impl Serialize for Labels {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for kv in self.0.iter() {
            seq.serialize_element(&kv)?;
        }
        seq.end()
    }
}

struct LabelsVisitor;

impl<'de> Visitor<'de> for LabelsVisitor {
    type Value = Labels;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A list of nodeid-label tuples")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut res = HashMap::new();
        while let Some((k, v)) = seq.next_element()? {
            res.insert(k, v);
        }
        Ok(Labels(res))
    }
}

impl<'de> Deserialize<'de> for Labels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(LabelsVisitor)
    }
}
