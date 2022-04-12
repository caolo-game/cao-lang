//! The public representation of a program
//!

use crate::compiler::Lane;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum IntoStreamError {
    #[error("Main function by name {0} was not found")]
    MainFnNotFound(String),
    #[error("{0:?} is not a valid name")]
    BadName(String),
}

pub type CaoProgram = Module;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module {
    #[cfg_attr(feature = "serde", serde(default = "HashMap::default"))]
    pub submodules: HashMap<String, Module>,
    pub lanes: HashMap<String, Lane>,
}

impl Module {
    /// flatten this program into a lane stream
    // TODO: return an iterator???
    pub(crate) fn into_ir_stream(mut self) -> Result<Vec<Lane>, CompilationErrorPayload> {
        // the first lane is special
        //
        let first_fn = self
            .lanes
            .remove("main")
            .ok_or(CompilationErrorPayload::NoMain)?;

        let mut result = vec![first_fn];
        result.reserve(self.lanes.len() * self.submodules.len() * 2); // just some dumb heuristic

        let mut namespace = SmallVec::<[_; 16]>::new();

        // flatten modules' functions
        flatten_module(&self, &mut namespace, &mut result)?;

        Ok(result)
    }
}

fn flatten_module<'a>(
    module: &'a Module,
    namespace: &mut SmallVec<[&'a str; 16]>,
    out: &mut Vec<Lane>,
) -> Result<(), CompilationErrorPayload> {
    for (name, submod) in module.submodules.iter() {
        namespace.push(name.as_str());
        flatten_module(submod, namespace, out)?;
        namespace.pop();
    }
    if out.capacity() - out.len() < module.lanes.len() {
        out.reserve(module.lanes.len() - (out.capacity() - out.len()));
    }
    for (name, lane) in module.lanes.iter() {
        if !is_name_valid(name.as_str()) {
            return Err(CompilationErrorPayload::BadLaneName(name.clone()));
        }
        let mut lane = lane.clone();
        namespace.push(name.as_str());
        lane.name = flatten_name(namespace.as_slice());
        namespace.pop();
        out.push(lane);
    }
    Ok(())
}

fn is_name_valid(name: &str) -> bool {
    !name.contains(|c: char| !c.is_alphanumeric() && c != '_') && !name.is_empty()
}

fn flatten_name(namespace: &[&str]) -> String {
    namespace.join(".")
}
