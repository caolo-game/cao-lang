//! The public representation of a program
//!

#[cfg(test)]
mod tests;

use crate::compiler::Function;
use crate::prelude::{CompilationErrorPayload, Handle};
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
    /// e.g. importing `foo.bar` allows you to use a `Jump("bar")` [Card]
    pub imports: Imports,
}

/// Uniquely index a card in a module
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardIndex {
    pub function: usize,
    pub card_index: FunctionCardIndex,
}

impl PartialOrd for CardIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.function.cmp(&other.function) {
            std::cmp::Ordering::Equal => {}
            c @ std::cmp::Ordering::Less | c @ std::cmp::Ordering::Greater => return c,
        }
        for (lhs, rhs) in self
            .card_index
            .indices
            .iter()
            .zip(other.card_index.indices.iter())
        {
            match lhs.cmp(&rhs) {
                std::cmp::Ordering::Equal => {}
                c @ std::cmp::Ordering::Less | c @ std::cmp::Ordering::Greater => return c,
            }
        }
        self.card_index
            .indices
            .len()
            .cmp(&other.card_index.indices.len())
    }
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
        self.push_subindex(card_index as u32);
        self
    }

    pub fn current_index(&self) -> usize {
        self.card_index.current_index()
    }

    /// Replaces the card index of the leaf node
    pub fn with_current_index(mut self, card_index: usize) -> Self {
        self.card_index.set_current_index(card_index);
        self
    }

    pub fn set_current_index(&mut self, card_index: usize) {
        self.card_index.set_current_index(card_index);
    }

    /// first card's index in the function
    pub fn begin(&self) -> Result<usize, CardFetchError> {
        self.card_index.begin()
    }

    /// Return wether this index points to a 'top level' card in the function.
    /// Instead of a nested card.
    pub fn is_top_level_card(&self) -> bool {
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
        self.push_sub_index(card_index);
        self
    }

    pub fn push_sub_index(&mut self, card_index: usize) {
        self.indices.push(card_index as u32);
    }

    #[must_use]
    pub fn current_index(&self) -> usize {
        self.indices.last().copied().unwrap_or(0) as usize
    }

    /// Replaces the card index of the leaf node
    #[must_use]
    pub fn with_current_index(mut self, card_index: usize) -> Self {
        self.set_current_index(card_index);
        self
    }

    pub fn set_current_index(&mut self, card_index: usize) {
        if let Some(x) = self.indices.last_mut() {
            *x = card_index as u32;
        }
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

#[derive(Debug, Clone, Error)]
pub enum SwapError {
    #[error("Failed to find card {0}: {1}")]
    FetchError(CardIndex, CardFetchError),
    #[error("These cards can not be swapped")]
    InvalidSwap,
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

        for (depth, i) in idx.card_index.indices[1..].iter().enumerate() {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: depth + 1 })?;
        }

        Ok(card)
    }

    pub fn get_card<'a>(&'a self, idx: &CardIndex) -> Result<&'a Card, CardFetchError> {
        let (_, function) = self
            .functions
            .get(idx.function)
            .ok_or(CardFetchError::FunctionNotFound)?;

        let mut depth = 0;
        let mut card = function
            .cards
            .get(idx.begin()?)
            .ok_or(CardFetchError::CardNotFound { depth })?;

        for i in &idx.card_index.indices[1..] {
            depth += 1;
            card = card
                .get_child(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth })?;
        }

        Ok(card)
    }

    /// swapping a parent and child is an error
    pub fn swap_cards<'a>(
        &mut self,
        mut lhs: &'a CardIndex,
        mut rhs: &'a CardIndex,
    ) -> Result<(), SwapError> {
        if lhs < rhs {
            std::mem::swap(&mut lhs, &mut rhs);
        }

        let rhs_card = self
            .replace_card(rhs, Card::ScalarNil)
            .map_err(|err| SwapError::FetchError(rhs.clone(), err))?;

        // check if lhs is reachable
        // run the check after taking rhs_card, as this can fail if lhs is a child of rhs
        if let Err(_) = self.get_card(lhs) {
            self.replace_card(rhs, rhs_card).unwrap();
            return Err(SwapError::InvalidSwap);
        }

        // we know that lhs is reachable so this mustn't err
        let lhs_card = self.replace_card(lhs, rhs_card).unwrap();

        // we know that rhs is reachable so this mustn't err
        self.replace_card(rhs, lhs_card).unwrap();
        Ok(())
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
        for (depth, i) in idx.card_index.indices[1..(len - 1).max(1)]
            .iter()
            .enumerate()
        {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: depth + 1 })?;
        }
        let i = *idx.card_index.indices.last().unwrap() as usize;
        card.remove_child(i)
            .ok_or(CardFetchError::CardNotFound { depth: len - 1 })
    }

    /// Return the old card
    pub fn replace_card(&mut self, idx: &CardIndex, child: Card) -> Result<Card, CardFetchError> {
        self.get_card_mut(idx).map(|c| std::mem::replace(c, child))
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
        for (depth, i) in idx.card_index.indices[1..(len - 1).max(1)]
            .iter()
            .enumerate()
        {
            card = card
                .get_child_mut(*i as usize)
                .ok_or(CardFetchError::CardNotFound { depth: depth + 1 })?;
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

        let mut result = Vec::with_capacity(self.functions.len() * self.submodules.len() * 2); // just some dumb heuristic

        let mut namespace = SmallVec::<[_; 16]>::new();

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
        for submodule_name in target.split('.') {
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
        for submodule_name in target.split('.') {
            current = current
                .submodules
                .iter_mut()
                .find(|(name, _)| name == submodule_name)
                .map(|(_, m)| m)?;
        }
        Some(current)
    }

    pub fn lookup_function(&self, target: &str) -> Option<&Function> {
        let Some((submodule, function)) = target.rsplit_once('.') else {
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
        let Some((submodule, function)) = target.rsplit_once('.') else {
            return self
                .functions
                .iter_mut()
                .find(|(name, _)| name == target)
                .map(|(_, l)| l);
        };
        let module = self.lookup_submodule_mut(submodule)?;
        module.lookup_function_mut(function)
    }

    /// Visits all cards in the module recursively
    ///
    /// ```
    /// use cao_lang::prelude::*;
    /// # use std::collections::HashSet;
    /// # use cao_lang::compiler::FunctionCardIndex;
    /// # use smallvec::smallvec;
    ///
    /// let mut program = CaoProgram {
    ///     imports: Default::default(),
    ///     submodules: Default::default(),
    ///     functions: [
    ///         (
    ///             "main".into(),
    ///             Function::default().with_card(Card::IfTrue(Box::new([
    ///                 Card::ScalarInt(42),
    ///                 Card::call_function("pooh", vec![]),
    ///             ]))),
    ///         ),
    ///         (
    ///             "pooh".into(),
    ///             Function::default().with_card(Card::set_global_var("result", Card::ScalarInt(69))),
    ///         ),
    ///     ]
    ///     .into(),
    /// };
    ///
    /// # let mut visited = HashSet::new();
    /// program.walk_cards_mut(|id, card| {
    ///     // use id, card
    /// #   visited.insert(id.clone());
    /// });
    ///
    /// # assert_eq!(visited.len(), 5);
    /// # let expected: HashSet<_> = [ CardIndex {
    /// #      function: 0,
    /// #      card_index: FunctionCardIndex {
    /// #          indices: smallvec![
    /// #              0,
    /// #              0,
    /// #          ],
    /// #      },
    /// #  },
    /// #  CardIndex {
    /// #      function: 0,
    /// #      card_index: FunctionCardIndex {
    /// #          indices: smallvec![
    /// #              0,
    /// #              1,
    /// #          ],
    /// #      },
    /// #  },
    /// #  CardIndex {
    /// #      function: 1,
    /// #      card_index: FunctionCardIndex {
    /// #          indices: smallvec![
    /// #              0,
    /// #          ],
    /// #      },
    /// #  },
    /// #  CardIndex {
    /// #      function: 1,
    /// #      card_index: FunctionCardIndex {
    /// #          indices: smallvec![
    /// #              0,
    /// #              0,
    /// #          ],
    /// #      },
    /// #  },
    /// #  CardIndex {
    /// #      function: 0,
    /// #      card_index: FunctionCardIndex {
    /// #          indices: smallvec![
    /// #              0,
    /// #          ],
    /// #      },
    /// #  },
    /// # ].into();
    /// # assert_eq!(visited, expected);
    /// ```
    pub fn walk_cards_mut(&mut self, mut op: impl FnMut(&CardIndex, &mut Card)) {
        let mut id = CardIndex::function(0);

        for (i, (_, f)) in self.functions.iter_mut().enumerate() {
            id.function = i;
            for (j, c) in f.cards.iter_mut().enumerate() {
                id.push_subindex(j as u32);
                op(&id, c);
                visit_children_mut(c, &mut id, &mut op);
                id.pop_subindex();
            }
        }
    }

    pub fn walk_cards(&mut self, mut op: impl FnMut(&CardIndex, &Card)) {
        let mut id = CardIndex::function(0);

        for (i, (_, f)) in self.functions.iter_mut().enumerate() {
            id.function = i;
            for (j, c) in f.cards.iter_mut().enumerate() {
                id.push_subindex(j as u32);
                op(&id, c);
                visit_children(c, &mut id, &mut op);
                id.pop_subindex();
            }
        }
    }
}

fn visit_children_mut(
    card: &mut Card,
    id: &mut CardIndex,
    op: &mut impl FnMut(&CardIndex, &mut Card),
) {
    id.push_subindex(0);
    for (k, child) in card.iter_children_mut().enumerate() {
        id.set_current_index(k);
        op(&id, child);
        visit_children_mut(child, id, op);
    }
    id.pop_subindex();
}

fn visit_children(card: &Card, id: &mut CardIndex, op: &mut impl FnMut(&CardIndex, &Card)) {
    id.push_subindex(0);
    for (k, child) in card.iter_children().enumerate() {
        id.set_current_index(k);
        op(&id, child);
        visit_children(child, id, op);
    }
    id.pop_subindex();
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
    for (function_id, (name, function)) in module.functions.iter().enumerate() {
        if !is_name_valid(name.as_ref()) {
            return Err(CompilationErrorPayload::BadFunctionName(name.to_string()));
        }
        namespace.push(name.as_ref());
        out.push(function_to_function_ir(
            out.len(),
            function_id,
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
    i: usize,
    function_id: usize,
    function: &Function,
    namespace: &[&str],
    imports: Rc<ImportsIr>,
) -> FunctionIr {
    assert!(
        !namespace.is_empty(),
        "Assume that function name is the last entry in namespace"
    );

    let mut cl = FunctionIr {
        function_index: function_id,
        name: namespace.last().unwrap().to_string().into_boxed_str(),
        arguments: function.arguments.clone().into_boxed_slice(),
        cards: function.cards.clone().into_boxed_slice(),
        imports,
        namespace: Default::default(),
        handle: Handle::from_u64(i as u64),
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
