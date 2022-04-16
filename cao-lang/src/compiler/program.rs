//! The public representation of a program
//!

use crate::compiler::Lane;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
use std::collections::HashMap;
use thiserror::Error;

use super::compiled_lane::CompiledLane;

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
    pub(crate) fn into_ir_stream(mut self) -> Result<Vec<CompiledLane>, CompilationErrorPayload> {
        // the first lane is special
        //
        let first_fn = self
            .lanes
            .remove("main")
            .ok_or(CompilationErrorPayload::NoMain)?;

        let first_fn = lane_to_compiled_lane(&first_fn, &["main"]);
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
    out: &mut Vec<CompiledLane>,
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
        namespace.push(name.as_str());
        out.push(lane_to_compiled_lane(lane, namespace));
        namespace.pop();
    }
    Ok(())
}

fn lane_to_compiled_lane(lane: &Lane, namespace: &[&str]) -> CompiledLane {
    assert!(
        !namespace.is_empty(),
        "Assume that lane name is the last entry in namespace"
    );

    let mut cl = CompiledLane {
        name: flatten_name(namespace),
        arguments: lane.arguments.clone(),
        cards: lane.cards.clone(),
        ..Default::default()
    };
    cl.namespace.extend(
        namespace
            .iter()
            .take(namespace.len() - 1)
            .map(|x| x.to_string()),
    );
    cl
}

fn is_name_valid(name: &str) -> bool {
    !name.contains(|c: char| !c.is_alphanumeric() && c != '_') && !name.is_empty()
}

fn flatten_name(namespace: &[&str]) -> String {
    namespace.join(".")
}
