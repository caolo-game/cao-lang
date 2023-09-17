use super::*;
use crate::InputString;
use crate::VarName;

impl Default for Card {
    fn default() -> Self {
        Card::ScalarNil
    }
}

/// Cao-Lang AST
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Card {
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
    /// Insert value at key into the table
    /// [Value, Table, Key]
    SetProperty(Box<[Card; 3]>),
    /// [Table, Key]
    GetProperty(BinaryExpression),
    ScalarInt(i64),
    ScalarFloat(f64),
    StringLiteral(String),
    CallNative(Box<CallNode>),
    /// Children = [condition, then]
    IfTrue(BinaryExpression),
    /// Children = [condition, else]
    IfFalse(BinaryExpression),
    /// Children = [condition, then, else]
    IfElse(Box<[Card; 3]>),
    /// Function name
    Call(Box<StaticJump>),
    /// Function name
    ///
    /// Creates a pointer to the given cao-lang function
    Function(String),
    /// Function name
    ///
    /// Creates a pointer to the given native function
    NativeFunction(String),
    SetGlobalVar(Box<SetVar>),
    SetVar(Box<SetVar>),
    ReadVar(VarName),
    /// repeats the `body` N times
    Repeat(Box<Repeat>),
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
    Closure(Box<Function>),
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
    pub function: Card,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticJump {
    pub args: Arguments,
    pub function_name: String,
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
            Card::CallNative(_) => "Call Native Function",
            Card::IfTrue(_) => "IfTrue",
            Card::IfFalse(_) => "IfFalse",
            Card::Call(_) => "Call Function",
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
            Card::NativeFunction(_) => "Native Function",
            Card::Closure(_) => "Closure",
        }
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

    pub fn repeat(n: Card, i: Option<String>, body: Card) -> Self {
        Self::Repeat(Box::new(Repeat { i, n, body }))
    }

    pub fn set_var(s: impl Into<String>, value: Card) -> Self {
        Self::SetVar(Box::new(SetVar {
            name: s.into(),
            value,
        }))
    }

    pub fn call_native(s: impl Into<InputString>, args: impl Into<Arguments>) -> Self {
        Self::CallNative(Box::new(CallNode {
            name: s.into(),
            args: args.into(),
        }))
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
            function_name: s.into(),
        }))
    }

    pub fn function_value(s: impl Into<String>) -> Self {
        Self::Function(s.into())
    }

    pub fn get_child_mut(&mut self, i: usize) -> Option<&mut Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get_mut(i)?,
            Card::Repeat(rep) => match i {
                0 => res = &mut rep.n,
                1 => res = &mut rep.body,
                _ => return None,
            },
            Card::IfTrue(c) | Card::IfFalse(c) => return c.get_mut(i),
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
            Card::IfElse(children) => return children.get_mut(i),

            Card::Add(expr)
            | Card::While(expr)
            | Card::Sub(expr)
            | Card::Mul(expr)
            | Card::Div(expr)
            | Card::Less(expr)
            | Card::LessOrEq(expr)
            | Card::Equals(expr)
            | Card::NotEquals(expr)
            | Card::And(expr)
            | Card::Or(expr)
            | Card::Xor(expr)
            | Card::AppendTable(expr)
            | Card::Get(expr)
            | Card::GetProperty(expr) => return expr.get_mut(i),
            Card::SetProperty(expr) => return expr.get_mut(i),

            Card::PopTable(expr) | Card::Not(expr) | Card::Return(expr) | Card::Len(expr) => {
                match i {
                    0 => res = &mut expr.card,
                    _ => return None,
                }
            }

            Card::SetGlobalVar(s) | Card::SetVar(s) => match i {
                0 => res = &mut s.value,
                _ => return None,
            },
            Card::CallNative(j) => return j.args.0.get_mut(i),
            Card::Call(j) => return j.args.0.get_mut(i),
            Card::DynamicCall(j) => {
                return (i == 0)
                    .then_some(&mut j.function)
                    .or_else(|| j.args.0.get_mut(i - 1))
            }
            Card::Array(cards) => return cards.get_mut(i),
            Card::Function(_)
            | Card::NativeFunction(_)
            | Card::Closure(_)
            | Card::ReadVar(_)
            | Card::ScalarInt(_)
            | Card::ScalarFloat(_)
            | Card::StringLiteral(_)
            | Card::ScalarNil
            | Card::CreateTable
            | Card::Abort => return None,
        }
        Some(res)
    }

    pub fn get_child(&self, i: usize) -> Option<&Card> {
        let res;
        match self {
            Card::CompositeCard(c) => res = c.cards.get(i)?,
            Card::Repeat(rep) => match i {
                0 => res = &rep.n,
                1 => res = &rep.body,
                _ => return None,
            },
            Card::IfTrue(c) | Card::IfFalse(c) => return c.get(i),
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
            Card::IfElse(children) => return children.get(i),
            Card::While(expr)
            | Card::Add(expr)
            | Card::Sub(expr)
            | Card::Mul(expr)
            | Card::Div(expr)
            | Card::Less(expr)
            | Card::LessOrEq(expr)
            | Card::Equals(expr)
            | Card::NotEquals(expr)
            | Card::And(expr)
            | Card::Or(expr)
            | Card::Xor(expr)
            | Card::AppendTable(expr)
            | Card::Get(expr)
            | Card::GetProperty(expr) => return expr.get(i),
            Card::SetProperty(expr) => return expr.get(i),

            Card::PopTable(expr) | Card::Not(expr) | Card::Return(expr) | Card::Len(expr) => {
                match i {
                    0 => res = &expr.card,
                    _ => return None,
                }
            }

            Card::SetGlobalVar(s) | Card::SetVar(s) => match i {
                0 => res = &s.value,
                _ => return None,
            },
            Card::CallNative(j) => return j.args.0.get(i),
            Card::Call(j) => return j.args.0.get(i),
            Card::DynamicCall(j) => {
                return (i == 0)
                    .then_some(&j.function)
                    .or_else(|| j.args.0.get(i - 1))
            }
            Card::Array(cards) => return cards.get(i),
            Card::Function(_)
            | Card::NativeFunction(_)
            | Card::Closure(_)
            | Card::ReadVar(_)
            | Card::ScalarInt(_)
            | Card::ScalarFloat(_)
            | Card::StringLiteral(_)
            | Card::ScalarNil
            | Card::CreateTable
            | Card::Abort => return None,
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
            Card::Repeat(rep) => match i {
                0 => res = std::mem::replace(&mut rep.n, Card::ScalarInt(0)),
                1 => res = std::mem::replace(&mut rep.body, Card::ScalarNil),
                _ => return None,
            },
            Card::IfTrue(_) | Card::IfFalse(_) => match self.get_child_mut(i) {
                Some(c) => {
                    res = std::mem::replace::<Card>(c, Card::ScalarNil);
                }
                None => return None,
            },

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
                    0 => res = std::mem::replace::<Card>(a.as_mut(), Card::ScalarNil),
                    1 => res = std::mem::replace::<Card>(b.as_mut(), Card::ScalarNil),
                    _ => return None,
                }
            }
            Card::IfElse(children) => {
                let Some(c) = children.get_mut(i) else {
                    return None;
                };
                res = std::mem::replace(c, Card::ScalarNil);
            }
            Card::While(_)
            | Card::Add(_)
            | Card::Sub(_)
            | Card::Mul(_)
            | Card::Div(_)
            | Card::Less(_)
            | Card::LessOrEq(_)
            | Card::Equals(_)
            | Card::NotEquals(_)
            | Card::And(_)
            | Card::Or(_)
            | Card::Xor(_)
            | Card::AppendTable(_)
            | Card::Get(_)
            | Card::SetProperty(_)
            | Card::PopTable(_)
            | Card::Not(_)
            | Card::Return(_)
            | Card::Len(_)
            | Card::SetGlobalVar(_)
            | Card::SetVar(_)
            | Card::GetProperty(_) => match self.get_child_mut(i) {
                Some(c) => res = std::mem::replace(c, Card::ScalarNil),
                None => return None,
            },

            Card::CallNative(j) => return (i < j.args.0.len()).then(|| j.args.0.remove(i)),
            Card::Call(j) => return (i < j.args.0.len()).then(|| j.args.0.remove(i)),
            Card::DynamicCall(j) => {
                if i == 0 {
                    res = std::mem::replace(&mut j.function, Card::ScalarNil);
                } else if i - 1 < j.args.0.len() {
                    res = j.args.0.remove(i - 1);
                } else {
                    return None;
                }
            }
            Card::Array(cards) => return (i < cards.len()).then(|| cards.remove(i)),
            Card::Function(_)
            | Card::NativeFunction(_)
            | Card::ReadVar(_)
            | Card::ScalarInt(_)
            | Card::ScalarFloat(_)
            | Card::StringLiteral(_)
            | Card::Closure(_)
            | Card::ScalarNil
            | Card::CreateTable
            | Card::Abort => return None,
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
            Card::IfElse(children) => match children.get_mut(i) {
                Some(c) => {
                    *c = card;
                }
                None => return Err(card),
            },
            Card::While(_)
            | Card::IfTrue(_)
            | Card::IfFalse(_)
            | Card::Add(_)
            | Card::Sub(_)
            | Card::Mul(_)
            | Card::Div(_)
            | Card::Less(_)
            | Card::LessOrEq(_)
            | Card::Equals(_)
            | Card::NotEquals(_)
            | Card::And(_)
            | Card::Or(_)
            | Card::Xor(_)
            | Card::AppendTable(_)
            | Card::Get(_)
            | Card::SetProperty(_)
            | Card::PopTable(_)
            | Card::Not(_)
            | Card::Return(_)
            | Card::Len(_)
            | Card::SetGlobalVar(_)
            | Card::SetVar(_)
            | Card::Repeat(_)
            | Card::GetProperty(_) => match self.get_child_mut(i) {
                Some(c) => *c = card,
                None => return Err(card),
            },
            Card::CallNative(j) => {
                (i <= j.args.0.len()).then(|| j.args.0.insert(i, card));
            }
            Card::Call(j) => {
                (i <= j.args.0.len()).then(|| j.args.0.insert(i, card));
            }
            Card::DynamicCall(j) => {
                if i == 0 {
                    j.function = card;
                } else if i - 1 <= j.args.0.len() {
                    j.args.0.insert(i - 1, card);
                } else {
                    return Err(card);
                }
            }

            Card::Array(children) => {
                if i <= children.len() {
                    children.insert(i, card);
                } else {
                    return Err(card);
                }
            }
            Card::Function(_)
            | Card::NativeFunction(_)
            | Card::ReadVar(_)
            | Card::Closure(_)
            | Card::ScalarInt(_)
            | Card::ScalarFloat(_)
            | Card::StringLiteral(_)
            | Card::ScalarNil
            | Card::CreateTable
            | Card::Abort => return Err(card),
        }
        Ok(())
    }

    /// Return Ok(old card) on success, return the input card in fail
    pub fn replace_child(&mut self, i: usize, card: Self) -> Result<Self, Self> {
        match self.get_child_mut(i) {
            Some(c) => Ok(std::mem::replace(c, card)),
            None => Err(card),
        }
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

    pub fn dynamic_call(function: Card, args: impl Into<Arguments>) -> Self {
        Self::DynamicCall(Box::new(DynamicJump {
            args: args.into(),
            function,
        }))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode {
    pub name: InputString,
    pub args: Arguments,
}

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

/// repeats the `body` N times
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Repeat {
    /// Loop variable is written into this variable
    pub i: Option<VarName>,
    pub n: Card,
    pub body: Card,
}
