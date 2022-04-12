use super::Card;
use crate::VarName;
use std::str::FromStr;

/// Cao-lang functions
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lane {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub arguments: Vec<VarName>,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub cards: Vec<Card>,
}

impl Lane {
    #[must_use]
    pub fn from_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
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
