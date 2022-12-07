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
    ScalarInt(i64),
    ScalarFloat(f64),
    StringLiteral(String),
    CallNative(Box<CallNode>),
    IfTrue(Box<Card>),
    IfFalse(Box<Card>),
    /// Children = [then, else]
    IfElse(Box<[Card; 2]>),
    /// Lane name
    Jump(String),
    SetGlobalVar(VarNode),
    SetVar(VarNode),
    ReadVar(VarNode),
    Repeat {
        /// Loop variable is written into this variable
        i: Option<VarNode>,
        body: Box<Card>,
    },
    /// Children = [condition, body]
    While(Box<[Card; 2]>),
    // TODO: move the entire variant into a struct
    ForEach {
        /// Loop variable is written into this variable
        i: Option<VarNode>,
        /// The key is written into this variable
        k: Option<VarNode>,
        /// The value is written into this variable
        v: Option<VarNode>,
        /// Variable that is iterated on
        variable: Box<Card>,
        body: Box<Card>,
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
            Card::Repeat { .. } => "Repeat",
            Card::While { .. } => "While",
            Card::IfElse { .. } => "IfElse",
            Card::GetProperty => "GetProperty",
            Card::SetProperty => "SetProperty",
            Card::ForEach { .. } => "ForEach",
            Card::CompositeCard(c) => c.ty.as_str(),
        }
    }

    /// Translate this Card into an Instruction, if possible.
    /// Some cards expand to multiple instructions, these are handled separately
    pub(crate) fn instruction(&self) -> Option<Instruction> {
        match self {
            Card::IfElse { .. }
            | Card::ReadVar(_)
            | Card::SetVar(_)
            | Card::While { .. }
            | Card::Repeat { .. }
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
            | Instruction::ForEach
            | Instruction::BeginForEach => {}
        };
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

    pub fn composite_card(ty: impl Into<String>, cards: Vec<Card>) -> Self {
        Self::CompositeCard(Box::new(CompositeCard {
            ty: ty.into(),
            cards,
        }))
    }

    pub fn set_var(s: &str) -> Result<Self, arrayvec::CapacityError> {
        let c = Self::SetVar(VarNode(Box::new(arrayvec::ArrayString::from_str(s)?)));
        Ok(c)
    }

    pub fn read_var(s: &str) -> Result<Self, arrayvec::CapacityError> {
        let c = Self::ReadVar(VarNode(Box::new(arrayvec::ArrayString::from_str(s)?)));
        Ok(c)
    }

    pub fn set_global_var(s: &str) -> Result<Self, arrayvec::CapacityError> {
        let c = Self::SetGlobalVar(VarNode(Box::new(arrayvec::ArrayString::from_str(s)?)));
        Ok(c)
    }

    pub fn scalar_int(i: i64) -> Self {
        Card::ScalarInt(i)
    }

    pub fn string_card(s: impl Into<String>) -> Self {
        Self::StringLiteral(s.into())
    }

    pub fn jump(s: impl Into<String>) -> Self {
        Self::Jump(s.into())
    }

    pub fn get_child_mut(&mut self, i: usize) -> Option<&mut Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get_mut(i)?,
            Card::Repeat { i: _, body: c } | Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = c;
            }
            Card::ForEach {
                i: _,
                k: _,
                v: _,
                variable: a,
                body: b,
            } => {
                if i > 1 {}
                match i {
                    0 => res = a.as_mut(),
                    1 => res = b.as_mut(),
                    _ => return None,
                }
            }
            Card::While(children) | Card::IfElse(children) => return children.get_mut(i),
            _ => return None,
        }
        Some(res)
    }

    pub fn get_child(&self, i: usize) -> Option<&Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get(i)?,
            Card::Repeat { i: _, body: c } | Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = c;
            }
            Card::ForEach {
                i: _,
                k: _,
                v: _,
                variable: a,
                body: b,
            } => {
                if i > 1 {}
                match i {
                    0 => res = a.as_ref(),
                    1 => res = b.as_ref(),
                    _ => return None,
                }
            }
            Card::While(children) | Card::IfElse(children) => return children.get(i),
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
            Card::Repeat { i: _, body: c } | Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return None;
                }
                res = std::mem::replace::<Card>(c.as_mut(), Card::Pass);
            }

            Card::ForEach {
                i: _,
                k: _,
                v: _,
                variable: a,
                body: b,
            } => {
                if i > 1 {}
                match i {
                    0 => res = std::mem::replace::<Card>(a.as_mut(), Card::Pass),
                    1 => res = std::mem::replace::<Card>(b.as_mut(), Card::Pass),
                    _ => return None,
                }
            }
            Card::While(children) | Card::IfElse(children) => {
                let Some(c) = children.get_mut(i) else {
                    return None
                };
                res = std::mem::replace(c, Card::Pass);
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
            Card::Repeat { i: _, body: c } | Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return Err(card);
                }
                *c.as_mut() = card;
            }

            Card::ForEach {
                i: _,
                k: _,
                v: _,
                variable: a,
                body: b,
            } => {
                if i > 1 {}
                match i {
                    0 => *a.as_mut() = card,
                    1 => *b.as_mut() = card,
                    _ => return Err(card),
                };
            }
            Card::While(children) | Card::IfElse(children) => {
                if let Some(c) = children.get_mut(i) {
                    *c = card;
                }
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
            Card::Repeat { i: _, body: c } | Card::IfTrue(c) | Card::IfFalse(c) => {
                if i != 0 {
                    return Err(card);
                }
                std::mem::replace(c.as_mut(), card)
            }
            Card::ForEach {
                i: _,
                k: _,
                v: _,
                variable: a,
                body: b,
            } => match i {
                0 => std::mem::replace(a.as_mut(), card),
                1 => std::mem::replace(b.as_mut(), card),
                _ => return Err(card),
            },
            Card::While(children) | Card::IfElse(children) => {
                let Some(c) = children.get_mut(i) else {
                    return Err(card);
                };
                std::mem::replace(c, card)
            }
            _ => return Err(card),
        };
        Ok(res)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode(pub InputString);

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
pub struct CompositeCard {
    /// Type is meant to be used by the implementation to store metadata
    pub ty: String,
    pub cards: Vec<Card>,
}
