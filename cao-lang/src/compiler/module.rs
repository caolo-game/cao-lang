//! The public representation of a program
//!

#[cfg(test)]
mod tests;

use crate::compiler::Lane;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hasher;
use std::rc::Rc;
use thiserror::Error;

use super::lane_ir::LaneIr;
use super::{Card, Imports};

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
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardIndex {
    pub lane: String,
    pub card_index: LaneCardIndex,
}

impl CardIndex {
    pub fn new(lane: &str, card_index: usize) -> Self {
        Self {
            lane: lane.to_owned(),
            card_index: LaneCardIndex::new(card_index),
        }
    }

    pub fn push_subindex(&mut self, i: u32) {
        self.card_index.indices.push(i);
    }

    pub fn pop_subindex(&mut self) {
        self.card_index.indices.pop();
    }

    pub fn as_handle(&self) -> crate::prelude::Handle {
        let mut handle = crate::prelude::Handle::from_bytes(self.lane.as_bytes());
        for i in self.card_index.indices.iter() {
            handle = handle + crate::prelude::Handle::from_u32(*i);
        }
        handle
    }

    /// pushes a new sub-index to the bottom layer
    #[must_use]
    pub fn with_sub_index(mut self, card_index: usize) -> Self {
        self.card_index = self.card_index.with_sub_index(card_index);
        self
    }

    pub fn current_index(&self) -> usize {
        self.card_index.current_index()
    }

    /// Replaces the card index of the leaf node
    pub fn with_current_index(mut self, card_index: usize) -> Self {
        self.card_index = self.card_index.with_current_index(card_index);
        self
    }

    /// first card's index in the lane
    pub fn begin(&self) -> Result<usize, CardFetchError> {
        self.card_index.begin()
    }

    /// Return wether this index points to a 'top level' card in the lane.
    /// Instead of a nested card.
    pub fn is_lane_card(&self) -> bool {
        self.card_index.indices.len() == 1
    }
}

impl std::fmt::Display for CardIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lane)?;
        for i in self.card_index.indices.iter() {
            write!(f, ".{}", i)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LaneCardIndex {
    pub indices: SmallVec<[u32; 4]>,
}

impl LaneCardIndex {
    #[must_use]
    pub fn new(card_index: usize) -> Self {
        Self {
            indices: smallvec::smallvec![card_index as u32],
        }
    }

    pub fn depth(&self) -> usize {
        self.indices.len()
    }

    /// pushes a new sub-index to the bottom layer
    #[must_use]
    pub fn with_sub_index(mut self, card_index: usize) -> Self {
        self.indices.push(card_index as u32);
        self
    }

    #[must_use]
    pub fn current_index(&self) -> usize {
        self.indices.last().copied().unwrap_or(0) as usize
    }

    /// Replaces the card index of the leaf node
    #[must_use]
    pub fn with_current_index(mut self, card_index: usize) -> Self {
        if let Some(x) = self.indices.last_mut() {
            *x = card_index as u32;
        }
        self
    }

    pub fn begin(&self) -> Result<usize, CardFetchError> {
        let i = self.indices.first().ok_or(CardFetchError::InvalidIndex)?;
        Ok(*i as usize)
    }
}

#[derive(Debug, Clone, Error)]
pub enum CardFetchError {
    #[error("Lane not found")]
    LaneNotFound,
    #[error("Card at depth {depth} not found")]
    CardNotFound { depth: usize },
    #[error("The card at depth {depth} has no nested lanes, but the index tried to fetch one")]
    NoSubLane { depth: usize },
    #[error("The provided index is not valid")]
    InvalidIndex,
}

impl Module {
    pub fn get_card_mut<'a>(&'a mut self, idx: &CardIndex) -> Result<&'a mut Card, CardFetchError> {
        let lane = self
            .lanes
            .get_mut(idx.lane.as_str())
            .ok_or(CardFetchError::LaneNotFound)?;
        let mut card = lane
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        for i in &idx.card_index.indices[1..] {
            card = card
                .get_card_by_index_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }

        Ok(card)
    }

    pub fn get_card<'a>(&'a self, idx: &CardIndex) -> Result<&'a Card, CardFetchError> {
        let lane = self
            .lanes
            .get(idx.lane.as_str())
            .ok_or(CardFetchError::LaneNotFound)?;
        let mut card = lane
            .cards
            .get(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        for i in &idx.card_index.indices[1..] {
            card = card
                .get_card_by_index(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }

        Ok(card)
    }

    pub fn remove_card(&mut self, idx: &CardIndex) -> Result<Card, CardFetchError> {
        let lane = self
            .lanes
            .get_mut(idx.lane.as_str())
            .ok_or(CardFetchError::LaneNotFound)?;
        if idx.card_index.indices.len() == 1 {
            if lane.cards.len() <= idx.card_index.indices[0] as usize {
                return Err(CardFetchError::CardNotFound { depth: 0 });
            }
            return Ok(lane.cards.remove(idx.card_index.indices[0] as usize));
        }
        let mut card = lane
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_card_by_index_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.remove_child(i)
            .ok_or(CardFetchError::CardNotFound { depth: len - 1 })
    }

    /// Return the old card
    pub fn replace_card(&mut self, idx: &CardIndex, child: Card) -> Result<Card, CardFetchError> {
        let lane = self
            .lanes
            .get_mut(idx.lane.as_str())
            .ok_or(CardFetchError::LaneNotFound)?;
        if idx.card_index.indices.len() == 1 {
            let c = lane
                .cards
                .get_mut(idx.card_index.indices[0] as usize)
                .ok_or(CardFetchError::CardNotFound { depth: 0 })?;
            let res = std::mem::replace(c, child);
            return Ok(res);
        }
        let mut card = lane
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_card_by_index_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.replace_child(i, child)
            .map_err(|_| CardFetchError::CardNotFound { depth: len - 1 })
    }

    pub fn insert_card(&mut self, idx: &CardIndex, child: Card) -> Result<(), CardFetchError> {
        let lane = self
            .lanes
            .get_mut(idx.lane.as_str())
            .ok_or(CardFetchError::LaneNotFound)?;
        if idx.card_index.indices.len() == 1 {
            if lane.cards.len() < idx.card_index.indices[0] as usize {
                return Err(CardFetchError::CardNotFound { depth: 0 });
            }
            lane.cards.insert(idx.card_index.indices[0] as usize, child);
            return Ok(());
        }
        let mut card = lane
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_card_by_index_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.insert_child(i, child)
            .map_err(|_| CardFetchError::CardNotFound { depth: len - 1 })
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
        let first_fn = lane_to_lane_ir(&first_fn, &["main"], Rc::new(imports));
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
        out.push(lane_to_lane_ir(lane, namespace, Rc::clone(&imports)));
        namespace.pop();
    }
    Ok(())
}

fn lane_to_lane_ir(lane: &Lane, namespace: &[&str], imports: Rc<Imports>) -> LaneIr {
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
