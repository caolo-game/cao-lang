use std::fmt::Display;

use super::*;
use crate::InputString;
use crate::VarName;

impl Default for Card {
    fn default() -> Self {
        Card::Pass
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Card {
    Pass,
    Add,
    Sub,
    Mul,
    Div,
    CopyLast,
    Less,
    LessOrEq,
    Equals,
    NotEquals,
    Pop,
    ClearStack,
    And,
    Or,
    Xor,
    Not,
    Return,
    ScalarNil,
    CreateTable,
    Abort,
    Len,
    SetProperty,
    GetProperty,
    ScalarInt(IntegerNode),
    ScalarFloat(FloatNode),
    StringLiteral(StringNode),
    CallNative(Box<CallNode>),
    IfTrue(Box<Card>),
    IfFalse(Box<Card>),
    IfElse {
        then: Box<Card>,
        r#else: Box<Card>,
    },
    Jump(LaneNode),
    SetGlobalVar(VarNode),
    SetVar(VarNode),
    ReadVar(VarNode),
    Repeat(LaneNode),
    While(LaneNode),
    ForEach {
        variable: VarNode,
        lane: LaneNode,
    },
    /// Single card that decomposes into multiple cards
    CompositeCard(Box<CompositeCard>),
}

impl Card {
    pub fn name(&self) -> &str {
        match self {
            Card::SetVar(_) => "SetLocalVar",
            Card::Pass => "Pass",
            Card::Add => "Add",
            Card::Sub => "Sub",
            Card::CreateTable => "CreateTable",
            Card::Mul => "Mul",
            Card::Div => "Div",
            Card::CopyLast => "CopyLast",
            Card::Not => "Not",
            Card::Less => "Less",
            Card::LessOrEq => "LessOrEq",
            Card::Equals => "Equals",
            Card::NotEquals => "NotEquals",
            Card::Pop => "Pop",
            Card::And => "And",
            Card::Or => "Either",
            Card::Xor => "Exclusive Or",
            Card::Abort => "Abort",
            Card::Len => "Len",
            Card::ScalarInt(_) => "ScalarInt",
            Card::ScalarFloat(_) => "ScalarFloat",
            Card::StringLiteral(_) => "StringLiteral",
            Card::CallNative(_) => "Call",
            Card::IfTrue(_) => "IfTrue",
            Card::IfFalse(_) => "IfFalse",
            Card::Jump(_) => "Jump",
            Card::SetGlobalVar(_) => "SetGlobalVar",
            Card::ReadVar(_) => "ReadVar",
            Card::ClearStack => "ClearStack",
            Card::ScalarNil => "ScalarNil",
            Card::Return => "Return",
            Card::Repeat(_) => "Repeat",
            Card::While(_) => "While",
            Card::IfElse { .. } => "IfElse",
            Card::GetProperty => "GetProperty",
            Card::SetProperty => "SetProperty",
            Card::ForEach { .. } => "ForEach",
            Card::CompositeCard(c) => c.name.as_str(),
        }
    }

    /// Translate this Card into an Instruction, if possible.
    /// Some cards expand to multiple instructions, these are handled separately
    pub(crate) fn instruction(&self) -> Option<Instruction> {
        match self {
            Card::IfElse { .. }
            | Card::ReadVar(_)
            | Card::SetVar(_)
            | Card::While(_)
            | Card::Repeat(_)
            | Card::ForEach { .. }
            | Card::Pass
            | Card::CompositeCard { .. } => None,

            Card::GetProperty => Some(Instruction::GetProperty),
            Card::SetProperty => Some(Instruction::SetProperty),
            Card::CreateTable => Some(Instruction::InitTable),
            Card::And => Some(Instruction::And),
            Card::Abort => Some(Instruction::Exit),
            Card::Not => Some(Instruction::Not),
            Card::Or => Some(Instruction::Or),
            Card::Xor => Some(Instruction::Xor),
            Card::Add => Some(Instruction::Add),
            Card::Sub => Some(Instruction::Sub),
            Card::Mul => Some(Instruction::Mul),
            Card::Div => Some(Instruction::Div),
            Card::CopyLast => Some(Instruction::CopyLast),
            Card::Less => Some(Instruction::Less),
            Card::LessOrEq => Some(Instruction::LessOrEq),
            Card::Equals => Some(Instruction::Equals),
            Card::NotEquals => Some(Instruction::NotEquals),
            Card::Pop => Some(Instruction::Pop),
            Card::ScalarInt(_) => Some(Instruction::ScalarInt),
            Card::ScalarFloat(_) => Some(Instruction::ScalarFloat),
            Card::CallNative(_) => Some(Instruction::Call),
            Card::IfTrue(_) => None,
            Card::IfFalse(_) => None,
            Card::Jump(_) => Some(Instruction::CallLane),
            Card::StringLiteral(_) => Some(Instruction::StringLiteral),
            Card::SetGlobalVar(_) => Some(Instruction::SetGlobalVar),
            Card::ClearStack => Some(Instruction::ClearStack),
            Card::ScalarNil => Some(Instruction::ScalarNil),
            Card::Return => Some(Instruction::Return),
            Card::Len => Some(Instruction::Len),
        }
    }

    // Trigger compilation errors for newly added instructions,
    // so we don't forget implementing a card for them
    #[allow(unused)]
    fn __instruction_to_node(instr: Instruction) {
        match instr {
            Instruction::SetGlobalVar
            | Instruction::Len
            | Instruction::ReadGlobalVar
            | Instruction::GetProperty
            | Instruction::SetProperty
            | Instruction::Pop
            | Instruction::Less
            | Instruction::LessOrEq
            | Instruction::Equals
            | Instruction::NotEquals
            | Instruction::Exit
            | Instruction::InitTable
            | Instruction::StringLiteral
            | Instruction::CallLane
            | Instruction::CopyLast
            | Instruction::Call
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::ClearStack
            | Instruction::ScalarFloat
            | Instruction::And
            | Instruction::Not
            | Instruction::Or
            | Instruction::Xor
            | Instruction::ScalarInt
            | Instruction::Add
            | Instruction::ScalarNil
            | Instruction::Return
            | Instruction::SwapLast
            | Instruction::ReadLocalVar
            | Instruction::SetLocalVar
            | Instruction::Goto
            | Instruction::GotoIfTrue
            | Instruction::GotoIfFalse
            | Instruction::Repeat
            | Instruction::BeginRepeat
            | Instruction::ForEach
            | Instruction::BeginForEach => {}
        };
    }

    /// If the card has sub-cards then return references to them in a list
    pub fn as_card_list(&self) -> Option<smallvec::SmallVec<[&Card; 4]>> {
        let mut res = smallvec::smallvec![];
        match self {
            Self::CompositeCard(c) => {
                res.extend(c.cards.iter());
            }
            Self::IfTrue(c) | Self::IfFalse(c) => res.push(c.as_ref()),
            Self::IfElse { then, r#else } => {
                res.push(then.as_ref());
                res.push(r#else.as_ref());
            }
            _ => return None,
        }
        Some(res)
    }

    /// If the card has sub-cards then return references to them in a list
    pub fn as_card_list_mut(&mut self) -> Option<smallvec::SmallVec<[&mut Card; 4]>> {
        let mut res = smallvec::smallvec![];
        match self {
            Self::CompositeCard(c) => {
                res.extend(c.cards.iter_mut());
            }
            Self::IfTrue(c) | Self::IfFalse(c) => res.push(c.as_mut()),
            Self::IfElse { then, r#else } => {
                res.push(then.as_mut());
                res.push(r#else.as_mut());
            }
            _ => return None,
        }
        Some(res)
    }

    pub fn as_composite_card(&self) -> Option<&CompositeCard> {
        if let Self::CompositeCard(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_composite_card_mut(&mut self) -> Option<&mut CompositeCard> {
        if let Self::CompositeCard(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn composite_card(name: String, ty: String, cards: Vec<Card>) -> Self {
        Self::CompositeCard(Box::new(CompositeCard { name, ty, cards }))
    }

    pub fn string_card(s: impl Into<String>) -> Self {
        Self::StringLiteral(StringNode(s.into()))
    }

    pub fn get_card_by_index_mut(&mut self, i: usize) -> Option<&mut Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get_mut(i)?,
            Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = c;
            }
            Card::IfElse { then, r#else } => {
                if i > 1 {}
                match i {
                    0 => res = then.as_mut(),
                    1 => res = r#else.as_mut(),
                    _ => return None,
                }
            }
            _ => return None,
        }
        Some(res)
    }

    pub fn get_card_by_index(&self, i: usize) -> Option<&Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get(i)?,
            Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = c;
            }
            Card::IfElse { then, r#else } => {
                if i > 1 {}
                match i {
                    0 => res = then.as_ref(),
                    1 => res = r#else.as_ref(),
                    _ => return None,
                }
            }
            _ => return None,
        }
        Some(res)
    }

    pub fn remove_child(&mut self, i: usize) -> Option<Card> {
        let res;
        match self {
            Card::CompositeCard(c) => {
                if c.cards.len() <= i {
                    return None;
                }
                res = c.cards.remove(i);
            }
            Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = std::mem::replace::<Card>(c.as_mut(), Card::Pass);
            }
            Card::IfElse { then, r#else } => {
                if i > 1 {}
                match i {
                    0 => res = std::mem::replace::<Card>(then.as_mut(), Card::Pass),
                    1 => res = std::mem::replace::<Card>(r#else.as_mut(), Card::Pass),
                    _ => return None,
                }
            }
            _ => return None,
        }
        Some(res)
    }

    /// insert a child at the specified index, if the Card is a list, or replace the child at the
    /// index if not
    ///
    /// returns the inserted card on failure
    pub fn insert_child(&mut self, i: usize, card: Self) -> Result<(), Self> {
        match self {
            Card::CompositeCard(c) => {
                if c.cards.len() < i {
                    return Err(card);
                }
                c.cards.insert(i, card);
            }
            Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return Err(card);
                }
                *c.as_mut() = card;
            }
            Card::IfElse { then, r#else } => {
                if i > 1 {}
                match i {
                    0 => *then.as_mut() = card,
                    1 => *r#else.as_mut() = card,
                    _ => return Err(card),
                };
            }
            _ => return Err(card),
        }
        Ok(())
    }

    /// Return Ok(old card) on success, return the input card in fail
    pub fn replace_child(&mut self, i: usize, card: Self) -> Result<Self, Self> {
        let res = match self {
            Card::CompositeCard(c) => match c.cards.get_mut(i) {
                Some(c) => std::mem::replace(c, card),
                None => return Err(card),
            },
            Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return Err(card);
                }
                std::mem::replace(c.as_mut(), card)
            }
            Card::IfElse { then, r#else } => match i {
                0 => std::mem::replace(then.as_mut(), card),
                1 => std::mem::replace(r#else.as_mut(), card),
                _ => return Err(card),
            },
            _ => return Err(card),
        };
        Ok(res)
    }
}

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode(pub i64);

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FloatNode(pub f64);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode(pub InputString);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubProgramNode(pub InputString);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringNode(pub String);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VarNode(pub Box<VarName>);

impl VarNode {
    /// panics if the string is too long
    pub fn from_str_unchecked(s: &str) -> Self {
        Self(Box::new(
            VarName::from(s).expect("Failed to parse variable name"),
        ))
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LaneNode(pub String);

impl Display for LaneNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<LaneNode> for Handle {
    fn from(ln: LaneNode) -> Self {
        Self::from(&ln)
    }
}

impl<'a> From<&'a LaneNode> for Handle {
    fn from(ln: &'a LaneNode) -> Self {
        Handle::from_str(ln.0.as_str()).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompositeCard {
    /// Name is meant to be used as the display name of the custom card
    pub name: String,
    /// Type is meant to be used internally for special types of cards in an application
    pub ty: String,
    pub cards: Vec<Card>,
}
