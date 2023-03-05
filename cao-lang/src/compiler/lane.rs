use super::Card;
use crate::VarName;
use std::str::FromStr;

/// Cao-lang functions
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Function {
    pub arguments: Vec<VarName>,
    pub cards: Vec<Card>,
}

impl Function {
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
