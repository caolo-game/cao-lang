use smallvec::SmallVec;

use super::Card;
use crate::VarName;
use std::str::FromStr;

/// Cao-lang functions
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lane {
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub arguments: Vec<VarName>,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub cards: Vec<Card>,
}

impl Lane {
    /// Return the state of the stack at the given card
    ///
    /// Returns empty list on invalid index
    pub fn compute_stack_at_card(&self, card_id: usize) -> SmallVec<[String; 8]> {
        let mut result = SmallVec::new();
        if card_id >= self.cards.len() {
            return result;
        }

        for _arg in self.arguments.iter() {
            result.push("Any".to_string());
        }

        for card in &self.cards[..card_id] {
            let subprog = super::card_description::get_desc(card);

            for _i in 0..subprog.input.len() {
                result.pop();
            }
            for out in subprog.output.iter() {
                result.push(out.clone());
            }
        }

        result
    }

    #[must_use]
    pub fn with_arg(mut self, name: &str) -> Self {
        let name = VarName::from_str(name).expect("Bad variable name");
        self.arguments.push(name);
        self
    }

    #[must_use]
    pub fn with_card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }

    /// overrides the existing cards
    #[must_use]
    pub fn with_cards<C: Into<Vec<Card>>>(mut self, cards: C) -> Self {
        self.cards = cards.into();
        self
    }
}
