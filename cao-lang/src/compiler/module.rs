//! The public representation of a program
//!

#[cfg(test)]
mod tests;

use crate::compiler::Function;
use crate::prelude::CompilationErrorPayload;
use smallvec::SmallVec;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::rc::Rc;
use thiserror::Error;

use super::function_ir::FunctionIr;
use super::{Card, ImportsIr};

#[derive(Debug, Clone, Error)]
pub enum IntoStreamError {
    #[error("Main function by name {0} was not found")]
    MainFnNotFound(String),
    #[error("{0:?} is not a valid name")]
    BadName(String),
}

pub type CaoProgram = Module;
pub type CaoIdentifier = String;
pub type Imports = Vec<CaoIdentifier>;
pub type Functions = Vec<(CaoIdentifier, Function)>;
pub type Submodules = Vec<(CaoIdentifier, Module)>;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module {
    pub submodules: Submodules,
    pub functions: Functions,
    /// _functions_ to import from submodules
    ///
    /// e.g. importing `foo.bar` allows you to use a `Jump("bar")` [[Card]]
    pub imports: Imports,
}

/// Uniquely index a card in a module
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardIndex {
    pub function: usize,
    pub card_index: FunctionCardIndex,
}

impl CardIndex {
    pub fn function(function: usize) -> Self {
        Self {
            function,
            ..Default::default()
        }
    }

    pub fn new(function: usize, card_index: usize) -> Self {
        Self {
            function,
            card_index: FunctionCardIndex::new(card_index),
        }
    }

    pub fn push_subindex(&mut self, i: u32) {
        self.card_index.indices.push(i);
    }

    pub fn pop_subindex(&mut self) {
        self.card_index.indices.pop();
    }

    pub fn as_handle(&self) -> crate::prelude::Handle {
        let function_handle = crate::prelude::Handle::from_u64(self.function as u64);
        let subindices = self.card_index.indices.as_slice();
        let sub_handle = unsafe {
            crate::prelude::Handle::from_bytes(std::slice::from_raw_parts(
                subindices.as_ptr().cast(),
                subindices.len() * 4,
            ))
        };
        function_handle + sub_handle
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

    /// first card's index in the function
    pub fn begin(&self) -> Result<usize, CardFetchError> {
        self.card_index.begin()
    }

    /// Return wether this index points to a 'top level' card in the function.
    /// Instead of a nested card.
    pub fn is_function_card(&self) -> bool {
        self.card_index.indices.len() == 1
    }
}

impl std::fmt::Display for CardIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)?;
        for i in self.card_index.indices.iter() {
            write!(f, ".{}", i)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionCardIndex {
    pub indices: SmallVec<[u32; 4]>,
}

impl FunctionCardIndex {
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
    #[error("Function not found")]
    FunctionNotFound,
    #[error("Card at depth {depth} not found")]
    CardNotFound { depth: usize },
    #[error("The card at depth {depth} has no nested functions, but the index tried to fetch one")]
    NoSubFunction { depth: usize },
    #[error("The provided index is not valid")]
    InvalidIndex,
}

impl Module {
    pub fn get_card_mut<'a>(&'a mut self, idx: &CardIndex) -> Result<&'a mut Card, CardFetchError> {
        let (_, function) = self
            .functions
            .get_mut(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;
        let mut card = function
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        for i in &idx.card_index.indices[1..] {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }

        Ok(card)
    }

    pub fn get_card<'a>(&'a self, idx: &CardIndex) -> Result<&'a Card, CardFetchError> {
        let (_, function) = self
            .functions
            .get(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;
        let mut card = function
            .cards
            .get(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        for i in &idx.card_index.indices[1..] {
            card = card
                .get_child(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }

        Ok(card)
    }

    pub fn remove_card(&mut self, idx: &CardIndex) -> Result<Card, CardFetchError> {
        let (_, function) = self
            .functions
            .get_mut(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;
        if idx.card_index.indices.len() == 1 {
            if function.cards.len() <= idx.card_index.indices[0] as usize {
                return Err(CardFetchError::CardNotFound { depth: 0 });
            }
            return Ok(function.cards.remove(idx.card_index.indices[0] as usize));
        }
        let mut card = function
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.remove_child(i)
            .ok_or(CardFetchError::CardNotFound { depth: len - 1 })
    }

    /// Return the old card
    pub fn replace_card(&mut self, idx: &CardIndex, child: Card) -> Result<Card, CardFetchError> {
        let (_, function) = self
            .functions
            .get_mut(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;
        if idx.card_index.indices.len() == 1 {
            let c = function
                .cards
                .get_mut(idx.card_index.indices[0] as usize)
                .ok_or(CardFetchError::CardNotFound { depth: 0 })?;
            let res = std::mem::replace(c, child);
            return Ok(res);
        }
        let mut card = function
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.replace_child(i, child)
            .map_err(|_| CardFetchError::CardNotFound { depth: len - 1 })
    }

    pub fn insert_card(&mut self, idx: &CardIndex, child: Card) -> Result<(), CardFetchError> {
        let (_, function) = self
            .functions
            .get_mut(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;
        if idx.card_index.indices.len() == 1 {
            if function.cards.len() < idx.card_index.indices[0] as usize {
                return Err(CardFetchError::CardNotFound { depth: 0 });
            }
            function
                .cards
                .insert(idx.card_index.indices[0] as usize, child);
            return Ok(());
        }
        let mut card = function
            .cards
            .get_mut(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth: 0 })?;

        // len is at least 1
        let len = idx.card_index.indices.len();
        for i in &idx.card_index.indices[1..(len - 1).max(1)] {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: *i as usize })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.insert_child(i, child)
            .map_err(|_| CardFetchError::CardNotFound { depth: len - 1 })
    }

    /// flatten this program into a vec of functions
    ///
    /// called on the root module
    pub(crate) fn into_ir_stream(
        mut self,
        recursion_limit: u32,
    ) -> Result<Vec<FunctionIr>, CompilationErrorPayload> {
        // inject the standard library
        self.submodules
            .push(("std".to_string(), crate::stdlib::standard_library()));

        self.ensure_invariants(&mut Default::default())?;
        // the first function is special
        //
        let (main_index, _) = self
            .functions
            .iter()
            .enumerate()
            .find(|(_, (name, _))| name == "main")
            .ok_or(CompilationErrorPayload::NoMain)?;

        let mut result = vec![];
        result.reserve(self.functions.len() * self.submodules.len() * 2); // just some dumb heuristic

        let mut namespace = SmallVec::<[_; 16]>::new();

        // flatten modules' functions
        flatten_module(&self, recursion_limit, &mut namespace, &mut result)?;

        // move the main function to the front
        result.swap(0, main_index);
        Ok(result)
    }

    fn ensure_invariants<'a>(
        &'a self,
        aux: &mut std::collections::HashSet<&'a str>,
    ) -> Result<(), CompilationErrorPayload> {
        // test that submodule names are unique
        for (name, _) in self.submodules.iter() {
            if aux.contains(name.as_str()) {
                return Err(CompilationErrorPayload::DuplicateModule(name.to_string()));
            }
            aux.insert(name.as_str());
        }
        for (_, module) in self.submodules.iter() {
            aux.clear();
            module.ensure_invariants(aux)?;
        }
        Ok(())
    }

    fn execute_imports(&self) -> Result<ImportsIr, CompilationErrorPayload> {
        let mut result = ImportsIr::with_capacity(self.imports.len());

        for import in self.imports.iter() {
            let import = import.as_str();

            match import.rsplit_once('.') {
                Some((_, name)) => {
                    if result.contains_key(name) {
                        return Err(CompilationErrorPayload::AmbigousImport(import.to_string()));
                    }
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
    /// Keys = functions, submodules, card names.
    pub fn compute_keys_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash_module(&mut hasher, self);
        hasher.finish()
    }

    pub fn lookup_submodule(&self, target: &str) -> Option<&Module> {
        let mut current = self;
        for submodule_name in target.split(".") {
            current = current
                .submodules
                .iter()
                .find(|(name, _)| name == submodule_name)
                .map(|(_, m)| m)?;
        }
        Some(current)
    }

    pub fn lookup_submodule_mut(&mut self, target: &str) -> Option<&mut Module> {
        let mut current = self;
        for submodule_name in target.split(".") {
            current = current
                .submodules
                .iter_mut()
                .find(|(name, _)| name == submodule_name)
                .map(|(_, m)| m)?;
        }
        Some(current)
    }

    pub fn lookup_function(&self, target: &str) -> Option<&Function> {
        let Some((submodule, function)) = target.rsplit_once(".") else {
            return self
                .functions
                .iter()
                .find(|(name, _)| name == target)
                .map(|(_, l)| l);
        };
        let module = self.lookup_submodule(submodule)?;
        module.lookup_function(function)
    }

    pub fn lookup_function_mut(&mut self, target: &str) -> Option<&mut Function> {
        let Some((submodule, function)) = target.rsplit_once(".") else {
            return self
                .functions
                .iter_mut()
                .find(|(name, _)| name == target)
                .map(|(_, l)| l);
        };
        let module = self.lookup_submodule_mut(submodule)?;
        module.lookup_function_mut(function)
    }
}

fn hash_module(hasher: &mut impl Hasher, module: &Module) {
    for (name, function) in module.functions.iter() {
        hasher.write(name.as_str().as_bytes());
        hash_function(hasher, function);
    }
    for (name, submodule) in module.submodules.iter() {
        hasher.write(name.as_str().as_bytes());
        hash_module(hasher, submodule);
    }
}

fn hash_function(hasher: &mut impl Hasher, function: &Function) {
    for card in function.cards.iter() {
        hasher.write(card.name().as_bytes());
    }
}

fn flatten_module<'a>(
    module: &'a Module,
    recursion_limit: u32,
    namespace: &mut SmallVec<[&'a str; 16]>,
    out: &mut Vec<FunctionIr>,
) -> Result<(), CompilationErrorPayload> {
    if namespace.len() >= recursion_limit as usize {
        return Err(CompilationErrorPayload::RecursionLimitReached(
            recursion_limit,
        ));
    }
    if out.capacity() - out.len() < module.functions.len() {
        out.reserve(module.functions.len() - (out.capacity() - out.len()));
    }
    let imports = Rc::new(module.execute_imports()?);
    for (name, function) in module.functions.iter() {
        if !is_name_valid(name.as_ref()) {
            return Err(CompilationErrorPayload::BadFunctionName(name.to_string()));
        }
        namespace.push(name.as_ref());
        out.push(function_to_function_ir(
            function,
            namespace,
            Rc::clone(&imports),
        ));
        namespace.pop();
    }
    for (name, submod) in module.submodules.iter() {
        namespace.push(name.as_ref());
        flatten_module(submod, recursion_limit, namespace, out)?;
        namespace.pop();
    }
    Ok(())
}

fn function_to_function_ir(
    function: &Function,
    namespace: &[&str],
    imports: Rc<ImportsIr>,
) -> FunctionIr {
    assert!(
        !namespace.is_empty(),
        "Assume that function name is the last entry in namespace"
    );

    let mut cl = FunctionIr {
        name: flatten_name(namespace).into_boxed_str(),
        arguments: function.arguments.clone().into_boxed_slice(),
        cards: function.cards.clone().into_boxed_slice(),
        imports,
        namespace: Default::default(),
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
