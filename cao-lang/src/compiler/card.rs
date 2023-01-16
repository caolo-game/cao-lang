use super::*;
use crate::InputString;
use crate::VarName;

impl Default for Card {
    fn default() -> Self {
        Card::Pass
    }
}

/// Cao-Lang AST
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Card {
    Pass,
    Add(BinaryExpression),
    Sub(BinaryExpression),
    Mul(BinaryExpression),
    Div(BinaryExpression),
    Less(BinaryExpression),
    LessOrEq(BinaryExpression),
    Equals(BinaryExpression),
    NotEquals(BinaryExpression),
    And(BinaryExpression),
    Or(BinaryExpression),
    Xor(BinaryExpression),
    Not(UnaryExpression),
    Return(UnaryExpression),
    ScalarNil,
    CreateTable,
    Abort,
    Len(UnaryExpression),
    /// Pop the table, key, value from the stack
    /// Insert value at key into the table
    ///
    /// [Value, Table, Key]
    SetProperty(Box<[Card; 3]>),
    /// [Table, Key]
    GetProperty(BinaryExpression),
    ScalarInt(i64),
    ScalarFloat(f64),
    StringLiteral(String),
    CallNative(CallNode),
    IfTrue(Box<Card>),
    IfFalse(Box<Card>),
    /// Children = [then, else]
    IfElse(Box<[Card; 2]>),
    /// Lane name
    Call(Box<StaticJump>),
    /// Lane name
    ///
    /// Creates a pointer to the given cao-lang function
    Function(String),
    SetGlobalVar(Box<SetVar>),
    SetVar(Box<SetVar>),
    ReadVar(VarName),
    /// Pops the stack for an Integer N and repeats the `body` N times
    Repeat {
        /// Loop variable is written into this variable
        i: Option<VarName>,
        body: Box<Card>,
    },
    /// Children = [condition, body]
    While(Box<[Card; 2]>),
    ForEach(Box<ForEach>),
    /// Single card that decomposes into multiple cards
    CompositeCard(Box<CompositeCard>),
    /// Jump to the function that's on the top of the stack
    DynamicCall(Box<DynamicJump>),
    /// Get the given integer row of a table
    /// [Table, Index]
    Get(BinaryExpression),
    /// [Value, Table]
    AppendTable(BinaryExpression),
    PopTable(UnaryExpression),
    /// Create a Table from the results of the provided cards
    Array(Vec<Card>),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetVar {
    pub name: VarName,
    pub value: Card,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arguments(pub Vec<Card>);

impl From<Vec<Card>> for Arguments {
    fn from(value: Vec<Card>) -> Self {
        Arguments(value)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicJump {
    pub args: Arguments,
    pub lane: Card,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticJump {
    pub args: Arguments,
    pub lane_name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForEach {
    /// Loop variable is written into this variable
    pub i: Option<VarName>,
    /// The key is written into this variable
    pub k: Option<VarName>,
    /// The value is written into this variable
    pub v: Option<VarName>,
    /// Table that is iterated on
    pub iterable: Box<Card>,
    pub body: Box<Card>,
}

impl From<ForEach> for Card {
    fn from(value: ForEach) -> Self {
        Card::ForEach(Box::new(value))
    }
}

impl Card {
    pub fn name(&self) -> &str {
        match self {
            Card::SetVar(_) => "SetLocalVar",
            Card::Pass => "Pass",
            Card::Add(_) => "Add",
            Card::Sub(_) => "Sub",
            Card::CreateTable => "CreateTable",
            Card::Mul(_) => "Mul",
            Card::Div(_) => "Div",
            Card::Not(_) => "Not",
            Card::Less(_) => "Less",
            Card::LessOrEq(_) => "LessOrEq",
            Card::Equals(_) => "Equals",
            Card::NotEquals(_) => "NotEquals",
            Card::And(_) => "And",
            Card::Or(_) => "Either",
            Card::Xor(_) => "Exclusive Or",
            Card::Abort => "Abort",
            Card::Len(_) => "Len",
            Card::ScalarInt(_) => "ScalarInt",
            Card::ScalarFloat(_) => "ScalarFloat",
            Card::StringLiteral(_) => "StringLiteral",
            Card::CallNative(_) => "Call",
            Card::IfTrue(_) => "IfTrue",
            Card::IfFalse(_) => "IfFalse",
            Card::Call(_) => "Jump",
            Card::SetGlobalVar(_) => "SetGlobalVar",
            Card::ReadVar(_) => "ReadVar",
            Card::ScalarNil => "ScalarNil",
            Card::Return(_) => "Return",
            Card::Repeat { .. } => "Repeat",
            Card::While { .. } => "While",
            Card::IfElse { .. } => "IfElse",
            Card::GetProperty(_) => "GetProperty",
            Card::SetProperty(_) => "SetProperty",
            Card::ForEach { .. } => "ForEach",
            Card::CompositeCard(c) => c.ty.as_str(),
            Card::Function(_) => "Function",
            Card::DynamicCall(_) => "Dynamic Jump",
            Card::Get(_) => "Get",
            Card::AppendTable(_) => "Append to Table",
            Card::PopTable(_) => "Pop from Table",
            Card::Array(_) => "Array",
        }
    }

    // TODO: eventually most cards will not trivially compile to a single instruction
    // At that point get rid of this function
    //
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
            | Card::CompositeCard { .. }
            | Card::Array(_) => None,

            Card::GetProperty(_) => None,
            Card::SetProperty(_) => None,
            Card::CreateTable => Some(Instruction::InitTable),
            Card::Abort => Some(Instruction::Exit),
            Card::And(_) => None,
            Card::Not(_) => None,
            Card::Or(_) => None,
            Card::Xor(_) => None,
            Card::Add(_) => None,
            Card::Sub(_) => None,
            Card::Mul(_) => None,
            Card::Div(_) => None,
            Card::Less(_) => None,
            Card::LessOrEq(_) => None,
            Card::Equals(_) => None,
            Card::NotEquals(_) => None,
            Card::ScalarInt(_) => Some(Instruction::ScalarInt),
            Card::ScalarFloat(_) => Some(Instruction::ScalarFloat),
            Card::Function(_) => Some(Instruction::FunctionPointer),
            Card::CallNative(_) => Some(Instruction::CallNative),
            Card::IfTrue(_) => None,
            Card::IfFalse(_) => None,
            Card::Call(_) => None,
            Card::StringLiteral(_) => Some(Instruction::StringLiteral),
            Card::SetGlobalVar(_) => None,
            Card::ScalarNil => Some(Instruction::ScalarNil),
            Card::Return(_) => None,
            Card::Len(_) => None,
            Card::DynamicCall(_) => None,
            Card::Get(_) => None,
            Card::AppendTable(_) => None,
            Card::PopTable(_) => None,
        }
    }

    // TODO: eventually most cards will not trivially compile to a single instruction
    // At that point get rid of this function
    //
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
            | Instruction::CallNative
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
            | Instruction::FunctionPointer
            | Instruction::BeginForEach
            | Instruction::AppendTable
            | Instruction::PopTable
            | Instruction::CopyLast
            | Instruction::NthRow => {}
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

    pub fn set_var(s: impl Into<String>, value: Card) -> Self {
        Self::SetVar(Box::new(SetVar {
            name: s.into(),
            value,
        }))
    }

    pub fn call_native(s: impl Into<InputString>) -> Self {
        Self::CallNative(CallNode(s.into()))
    }

    pub fn read_var(s: impl Into<String>) -> Self {
        Self::ReadVar(s.into())
    }

    pub fn set_global_var(s: impl Into<String>, value: Card) -> Self {
        Self::SetGlobalVar(Box::new(SetVar {
            name: s.into(),
            value,
        }))
    }

    pub fn scalar_int(i: i64) -> Self {
        Card::ScalarInt(i)
    }

    pub fn string_card(s: impl Into<String>) -> Self {
        Self::StringLiteral(s.into())
    }

    pub fn call_function(s: impl Into<String>, args: impl Into<Arguments>) -> Self {
        Self::Call(Box::new(StaticJump {
            args: args.into(),
            lane_name: s.into(),
        }))
    }

    pub fn function_value(s: impl Into<String>) -> Self {
        Self::Function(s.into())
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
            Card::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
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
            Card::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_ref();
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

            Card::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
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

            Card::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
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
            Card::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
                match i {
                    0 => std::mem::replace(a.as_mut(), card),
                    1 => std::mem::replace(b.as_mut(), card),
                    _ => return Err(card),
                }
            }
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

    pub fn return_card(c: Self) -> Self {
        Card::Return(UnaryExpression { card: Box::new(c) })
    }

    pub fn set_property(value: Self, table: Self, key: Self) -> Self {
        Card::SetProperty(Box::new([value, table, key]))
    }

    pub fn get_property(table: Self, key: Self) -> Self {
        Card::GetProperty(Box::new([table, key]))
    }

    pub fn dynamic_call(lane: Card, args: impl Into<Arguments>) -> Self {
        Self::DynamicCall(Box::new(DynamicJump {
            args: args.into(),
            lane,
        }))
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode(pub InputString);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompositeCard {
    /// Type is meant to be used by the implementation to store metadata
    pub ty: String,
    pub cards: Vec<Card>,
}

impl From<CompositeCard> for Card {
    fn from(value: CompositeCard) -> Self {
        Card::CompositeCard(Box::new(value))
    }
}

pub type BinaryExpression = Box<[Card; 2]>;

// Some serialization format, like YAML doesn't support nesting Cards,
// so we need a named member
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnaryExpression {
    pub card: Box<Card>,
}

impl UnaryExpression {
    pub fn new(c: impl Into<Box<Card>>) -> Self {
        Self { card: c.into() }
    }
}
