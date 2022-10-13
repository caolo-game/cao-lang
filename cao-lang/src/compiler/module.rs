//! The public representation of a program
//!

use crate::compiler::Lane;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
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

pub type CaoProgram = Module;
pub type CaoIdentifier = String;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module {
    pub submodules: BTreeMap<CaoIdentifier, Module>,
    pub lanes: BTreeMap<CaoIdentifier, Lane>,
    /// _lanes_ to import from submodules
    ///
    /// e.g. importing `foo.bar` allows you to use a `Jump("bar")` [[Card]]
    pub imports: BTreeSet<CaoIdentifier>,
}

/// Uniquely index a card in a module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardIndex<'a> {
    pub lane: &'a str,
    pub card_index: LaneCardIndex,
}

impl<'a> CardIndex<'a> {
    pub fn new(lane: &'a str) -> Self {
        Self {
            lane,
            card_index: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct LaneCardIndex {
    pub card_index: usize,
    pub sub_card_index: Option<Box<LaneCardIndex>>,
}

impl LaneCardIndex {
    #[must_use]
    pub fn new(card_index: usize) -> Self {
        Self {
            card_index,
            sub_card_index: None,
        }
    }

    #[must_use]
    pub fn with_sub_index(mut self, card_index: usize) -> Self {
        self.sub_card_index = Some(Box::new(Self {
            card_index,
            sub_card_index: None,
        }));
        self
    }
}

#[derive(Debug, Clone, Error)]
pub enum CardFetchError {
    #[error("Lane not found")]
    LaneNotFound,
    #[error("Card not found")]
    CardNotFound,
    #[error("The card has no nested lanes, but the index tried to fetch one")]
    NoSubLane,
}

impl Module {
    pub fn get_card_mut<'a>(
        &'a mut self,
        idx: &CardIndex,
    ) -> Result<&'a mut super::Card, CardFetchError> {
        let lane = self
            .lanes
            .get_mut(idx.lane)
            .ok_or(CardFetchError::LaneNotFound)?;
        let mut card = lane
            .cards
            .get_mut(idx.card_index.card_index)
            .ok_or(CardFetchError::CardNotFound)?;

        let mut card_idx = idx.card_index.sub_card_index.as_ref();
        while let Some(sub_card_idx) = card_idx {
            card = card
                .as_composite_card_mut()
                .ok_or(CardFetchError::NoSubLane)?
                .cards
                .get_mut(sub_card_idx.card_index)
                .ok_or(CardFetchError::CardNotFound)?;
            card_idx = sub_card_idx.sub_card_index.as_ref();
        }
        Ok(card)
    }

    pub fn get_card<'a>(&'a self, idx: &CardIndex) -> Result<&'a super::Card, CardFetchError> {
        let lane = self
            .lanes
            .get(idx.lane)
            .ok_or(CardFetchError::LaneNotFound)?;
        let mut card = lane
            .cards
            .get(idx.card_index.card_index)
            .ok_or(CardFetchError::CardNotFound)?;

        let mut card_idx = idx.card_index.sub_card_index.as_ref();
        while let Some(sub_card_idx) = card_idx {
            card = card
                .as_composite_card()
                .ok_or(CardFetchError::NoSubLane)?
                .cards
                .get(sub_card_idx.card_index)
                .ok_or(CardFetchError::CardNotFound)?;
            card_idx = sub_card_idx.sub_card_index.as_ref();
        }
        Ok(card)
    }

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
            let import = import.as_str();

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
        hasher.write(name.as_str().as_bytes());
        hash_lane(hasher, lane);
    }
    for (name, submodule) in module.submodules.iter() {
        hasher.write(name.as_str().as_bytes());
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

        let default_prog = prog();
        let mut pl = vec![];
        let mut s = bincode::Serializer::new(&mut pl, DefaultOptions::new());
        default_prog.serialize(&mut s).unwrap();

        let mut reader =
            bincode::de::Deserializer::from_slice(pl.as_slice(), DefaultOptions::new());

        let _prog = Module::deserialize(&mut reader).unwrap();
    }

    fn prog() -> Module {
        use crate::compiler::{Card, StringNode};

        let mut lanes = BTreeMap::new();
        lanes.insert(
            "main".into(),
            Lane::default().with_card(Card::CompositeCard(Box::new(
                crate::compiler::CompositeCard {
                    name: "triplepog".to_string().into(),
                    cards: vec![
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                        Card::StringLiteral(StringNode("poggers".to_owned())),
                    ],
                },
            ))),
        );
        let default_prog = CaoProgram {
            imports: Default::default(),
            submodules: Default::default(),
            lanes,
        };
        default_prog
    }

    #[test]
    #[cfg(feature = "serde")]
    fn module_json_serde_test() {
        let default_prog = prog();
        let pl = serde_json::to_string_pretty(&default_prog).unwrap();

        let _prog: Module = serde_json::from_str(&pl).unwrap();
    }

    #[test]
    #[cfg(feature = "serde")]
    fn can_parse_json_test() {
        let json = r#"
        {
            "submodules": {},
            "imports": [],
            "lanes": {"main": {
                "name": "main",
                "arguments": [],
                "cards": [ {"Jump": "42" } ]
            }}
        }
"#;
        let _prog: Module = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn module_card_fetch_test() {
        let m = prog();

        let comp_card = m
            .get_card(&CardIndex {
                lane: "main",
                card_index: LaneCardIndex {
                    card_index: 0,
                    sub_card_index: None,
                },
            })
            .expect("failed to fetch card");

        assert!(matches!(
            comp_card,
            super::super::Card::CompositeCard { .. }
        ));

        let nested_card = m
            .get_card(&CardIndex {
                lane: "main",
                card_index: LaneCardIndex {
                    card_index: 0,
                    sub_card_index: Some(Box::new(LaneCardIndex {
                        card_index: 1,
                        sub_card_index: None,
                    })),
                },
            })
            .expect("failed to fetch nested card");

        assert!(matches!(nested_card, super::super::Card::StringLiteral(_)));
    }
}
