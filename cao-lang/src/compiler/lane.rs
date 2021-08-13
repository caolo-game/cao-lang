use super::Card;
use crate::VarName;
use std::str::FromStr;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lane {
    pub name: Option<String>,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub arguments: Vec<VarName>,
    #[cfg_attr(feature = "serde", serde(default = "Vec::new"))]
    pub cards: Vec<Card>,
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            name: None,
            arguments: Vec::new(),
            cards: Vec::new(),
        }
    }
}

impl Lane {
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_arg(mut self, name: &str) -> Self {
        let name = VarName::from_str(name).expect("Bad variable name");
        self.arguments.push(name);
        self
    }

    pub fn with_card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }

    /// overrides the existing cards
    pub fn with_cards<C: Into<Vec<Card>>>(mut self, cards: C) -> Self {
        self.cards = cards.into();
        self
    }
}
