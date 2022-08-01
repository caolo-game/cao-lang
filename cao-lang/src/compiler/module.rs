//! The public representation of a program
//!

use crate::compiler::Lane;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hasher;
use std::rc::Rc;
use thiserror::Error;

use super::lane_ir::LaneIr;
use super::Imports;

#[derive(Debug, Clone, Error)]
pub enum IntoStreamError {
    #[error("Main function by name {0} was not found")]
    MainFnNotFound(String),
    #[error("{0:?} is not a valid name")]
    BadName(String),
}

pub type CaoProgram<'a> = Module<'a>;
pub type CaoIdentifier<'a> = Cow<'a, str>;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module<'a> {
    pub submodules: BTreeMap<CaoIdentifier<'a>, Module<'a>>,
    pub lanes: BTreeMap<CaoIdentifier<'a>, Lane>,
    /// _lanes_ to import from submodules
    ///
    /// e.g. importing `foo.bar` allows you to use a `Jump("bar")` [[Card]]
    pub imports: BTreeSet<CaoIdentifier<'a>>,
}

impl<'a> Module<'a> {
    /// flatten this program into a vec of lanes
    pub(crate) fn into_ir_stream(
        mut self,
        recursion_limit: u32,
    ) -> Result<Vec<LaneIr>, CompilationErrorPayload> {
        // the first lane is special
        //
        let first_fn = self
            .lanes
            .remove("main")
            .ok_or(CompilationErrorPayload::NoMain)?;

        let imports = self.execute_imports()?;
        let first_fn = lane_to_compiled_lane(&first_fn, &["main"], Rc::new(imports));
        let mut result = vec![first_fn];
        result.reserve(self.lanes.len() * self.submodules.len() * 2); // just some dumb heuristic

        let mut namespace = SmallVec::<[_; 16]>::new();

        // flatten modules' functions
        flatten_module(&self, recursion_limit, &mut namespace, &mut result)?;

        Ok(result)
    }

    fn execute_imports(&self) -> Result<Imports, CompilationErrorPayload> {
        let mut result = Imports::with_capacity(self.imports.len());

        for import in self.imports.iter() {
            let import = import.as_ref();

            match import.rsplit_once('.') {
                Some((_, name)) => {
                    result.insert(name.to_string(), import.to_string());
                }
                None => {
                    return Err(CompilationErrorPayload::BadImport(import.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// Hash the keys in the program.
    ///
    /// Keys = lanes, submodules, card names.
    pub fn compute_keys_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash_module(&mut hasher, self);
        hasher.finish()
    }
}

fn hash_module(hasher: &mut impl Hasher, module: &Module) {
    for (name, lane) in module.lanes.iter() {
        hasher.write(name.as_ref().as_bytes());
        hash_lane(hasher, lane);
    }
    for (name, submodule) in module.submodules.iter() {
        hasher.write(name.as_ref().as_bytes());
        hash_module(hasher, submodule);
    }
}

fn hash_lane(hasher: &mut impl Hasher, lane: &Lane) {
    for card in lane.cards.iter() {
        hasher.write(card.name().as_bytes());
    }
}

fn flatten_module<'a>(
    module: &'a Module,
    recursion_limit: u32,
    namespace: &mut SmallVec<[&'a str; 16]>,
    out: &mut Vec<LaneIr>,
) -> Result<(), CompilationErrorPayload> {
    if namespace.len() >= recursion_limit as usize {
        return Err(CompilationErrorPayload::RecursionLimitReached(
            recursion_limit,
        ));
    }
    for (name, submod) in module.submodules.iter() {
        namespace.push(name.as_ref());
        flatten_module(submod, recursion_limit, namespace, out)?;
        namespace.pop();
    }
    if out.capacity() - out.len() < module.lanes.len() {
        out.reserve(module.lanes.len() - (out.capacity() - out.len()));
    }
    let imports = Rc::new(module.execute_imports()?);
    for (name, lane) in module.lanes.iter() {
        if !is_name_valid(name.as_ref()) {
            return Err(CompilationErrorPayload::BadLaneName(name.to_string()));
        }
        namespace.push(name.as_ref());
        out.push(lane_to_compiled_lane(lane, namespace, Rc::clone(&imports)));
        namespace.pop();
    }
    Ok(())
}

fn lane_to_compiled_lane(lane: &Lane, namespace: &[&str], imports: Rc<Imports>) -> LaneIr {
    assert!(
        !namespace.is_empty(),
        "Assume that lane name is the last entry in namespace"
    );

    let mut cl = LaneIr {
        name: flatten_name(namespace).into_boxed_str(),
        arguments: lane.arguments.clone().into_boxed_slice(),
        cards: lane.cards.clone().into_boxed_slice(),
        imports,
        ..Default::default()
    };
    cl.namespace.extend(
        namespace
            .iter()
            .take(namespace.len() - 1)
            .map(|x| x.to_string().into_boxed_str()),
    );
    cl
}

fn is_name_valid(name: &str) -> bool {
    !name.contains(|c: char| !c.is_alphanumeric() && c != '_')
        && !name.is_empty()
        && name != "super" // `super` is a reserved identifier
}

fn flatten_name(namespace: &[&str]) -> String {
    namespace.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn module_bincode_serde_test() {
        use bincode::DefaultOptions;
        use serde::{Deserialize, Serialize};

        use crate::compiler::{Card, StringNode};

        let mut lanes = BTreeMap::new();
        lanes.insert(
            "main".into(),
            Lane::default().with_card(Card::CompositeCard {
                name: "triplepog".to_string().into(),
                cards: vec![
                    Card::StringLiteral(StringNode("poggers".to_owned())),
                    Card::StringLiteral(StringNode("poggers".to_owned())),
                    Card::StringLiteral(StringNode("poggers".to_owned())),
                ],
            }),
        );
        let default_prog = CaoProgram {
            imports: Default::default(),
            submodules: Default::default(),
            lanes,
        };

        let mut pl = vec![];
        let mut s = bincode::Serializer::new(&mut pl, DefaultOptions::new());
        default_prog.serialize(&mut s).unwrap();

        let mut reader =
            bincode::de::Deserializer::from_slice(pl.as_slice(), DefaultOptions::new());

        let _prog = Module::deserialize(&mut reader).unwrap();
    }
}
