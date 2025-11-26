use std::sync::atomic::AtomicU64;

use super::*;
use crate::InputString;
use crate::VarName;

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardId(pub u64);

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Card {
    #[cfg_attr(feature = "serde", serde(skip, default = "random_id"))]
    pub id: CardId,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub body: CardBody,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn random_id() -> CardId {
    CardId(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
}

impl From<CardBody> for Card {
    fn from(value: CardBody) -> Self {
        Self {
            id: random_id(),
            body: value,
        }
    }
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardBody {
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
    #[default]
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
    Comment(String),
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetVar {
    pub name: VarName,
    pub value: Card,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arguments(pub Vec<Card>);

impl From<Vec<Card>> for Arguments {
    fn from(value: Vec<Card>) -> Self {
        Arguments(value)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicJump {
    pub args: Arguments,
    pub function: Card,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StaticJump {
    pub args: Arguments,
    pub function_name: String,
}

#[derive(Debug, Default, Clone)]
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
        CardBody::ForEach(Box::new(value)).into()
    }
}

impl Card {
    pub fn name(&self) -> &str {
        match &self.body {
            CardBody::SetVar(_) => "SetVar",
            CardBody::Add(_) => "Add",
            CardBody::Sub(_) => "Sub",
            CardBody::CreateTable => "CreateTable",
            CardBody::Mul(_) => "Mul",
            CardBody::Div(_) => "Div",
            CardBody::Not(_) => "Not",
            CardBody::Less(_) => "Less",
            CardBody::LessOrEq(_) => "LessOrEq",
            CardBody::Equals(_) => "Equals",
            CardBody::NotEquals(_) => "NotEquals",
            CardBody::And(_) => "And",
            CardBody::Or(_) => "Either",
            CardBody::Xor(_) => "Exclusive Or",
            CardBody::Abort => "Abort",
            CardBody::Len(_) => "Len",
            CardBody::ScalarInt(_) => "ScalarInt",
            CardBody::ScalarFloat(_) => "ScalarFloat",
            CardBody::StringLiteral(_) => "StringLiteral",
            CardBody::CallNative(_) => "Call Native Function",
            CardBody::IfTrue(_) => "IfTrue",
            CardBody::IfFalse(_) => "IfFalse",
            CardBody::Call(_) => "Call Function",
            CardBody::SetGlobalVar(_) => "SetGlobalVar",
            CardBody::ReadVar(_) => "ReadVar",
            CardBody::ScalarNil => "ScalarNil",
            CardBody::Return(_) => "Return",
            CardBody::Repeat { .. } => "Repeat",
            CardBody::While { .. } => "While",
            CardBody::IfElse { .. } => "IfElse",
            CardBody::GetProperty(_) => "GetProperty",
            CardBody::SetProperty(_) => "SetProperty",
            CardBody::ForEach { .. } => "ForEach",
            CardBody::CompositeCard(c) => c.ty.as_str(),
            CardBody::Function(_) => "Function",
            CardBody::DynamicCall(_) => "Call",
            CardBody::Get(_) => "Get",
            CardBody::AppendTable(_) => "Append to Table",
            CardBody::PopTable(_) => "Pop from Table",
            CardBody::Array(_) => "Array",
            CardBody::NativeFunction(_) => "Native Function",
            CardBody::Closure(_) => "Closure",
            CardBody::Comment(_) => "Comment",
        }
    }

    pub fn as_composite_card(&self) -> Option<&CompositeCard> {
        if let CardBody::CompositeCard(v) = &self.body {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_composite_card_mut(&mut self) -> Option<&mut CompositeCard> {
        if let CardBody::CompositeCard(v) = &mut self.body {
            Some(v)
        } else {
            None
        }
    }

    pub fn composite_card(ty: impl Into<String>, cards: Vec<Card>) -> Self {
        CardBody::CompositeCard(Box::new(CompositeCard {
            ty: ty.into(),
            cards,
        }))
        .into()
    }

    pub fn repeat(n: impl Into<Card>, i: Option<String>, body: impl Into<Card>) -> Self {
        CardBody::Repeat(Box::new(Repeat {
            i,
            n: n.into(),
            body: body.into(),
        }))
        .into()
    }

    pub fn set_var(s: impl Into<String>, value: impl Into<Card>) -> Self {
        CardBody::SetVar(Box::new(SetVar {
            name: s.into(),
            value: value.into(),
        }))
        .into()
    }

    pub fn call_native(s: impl Into<InputString>, args: impl Into<Arguments>) -> Self {
        CardBody::CallNative(Box::new(CallNode {
            name: s.into(),
            args: args.into(),
        }))
        .into()
    }

    pub fn read_var(s: impl Into<String>) -> Self {
        CardBody::ReadVar(s.into()).into()
    }

    pub fn set_global_var(s: impl Into<String>, value: impl Into<Card>) -> Self {
        CardBody::SetGlobalVar(Box::new(SetVar {
            name: s.into(),
            value: value.into(),
        }))
        .into()
    }

    pub fn scalar_int(i: i64) -> Self {
        CardBody::ScalarInt(i).into()
    }

    pub fn string_card(s: impl Into<String>) -> Self {
        CardBody::StringLiteral(s.into()).into()
    }

    pub fn call_function(s: impl Into<String>, args: impl Into<Arguments>) -> Self {
        CardBody::Call(Box::new(StaticJump {
            args: args.into(),
            function_name: s.into(),
        }))
        .into()
    }

    pub fn function_value(s: impl Into<String>) -> Self {
        CardBody::Function(s.into()).into()
    }

    pub fn num_children(&self) -> u32 {
        match &self.body {
            CardBody::Add(_b)
            | CardBody::Sub(_b)
            | CardBody::Mul(_b)
            | CardBody::Div(_b)
            | CardBody::Less(_b)
            | CardBody::LessOrEq(_b)
            | CardBody::Equals(_b)
            | CardBody::NotEquals(_b)
            | CardBody::And(_b)
            | CardBody::Or(_b)
            | CardBody::GetProperty(_b)
            | CardBody::IfTrue(_b)
            | CardBody::IfFalse(_b)
            | CardBody::While(_b)
            | CardBody::Get(_b)
            | CardBody::AppendTable(_b)
            | CardBody::Xor(_b) => 2,
            CardBody::PopTable(UnaryExpression { .. })
            | CardBody::Len(UnaryExpression { .. })
            | CardBody::Not(UnaryExpression { .. })
            | CardBody::Return(UnaryExpression { .. }) => 1,
            CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::Function(_)
            | CardBody::CreateTable
            | CardBody::ReadVar(_)
            | CardBody::NativeFunction(_)
            | CardBody::Abort
            | CardBody::ScalarNil => 0,
            CardBody::IfElse(_t) | CardBody::SetProperty(_t) => 3,
            CardBody::CallNative(c) => c.args.0.len() as u32,
            CardBody::Call(c) => c.args.0.len() as u32,
            CardBody::SetGlobalVar(_s) | CardBody::SetVar(_s) => 1,
            CardBody::Repeat(_r) => 2,
            CardBody::ForEach(_f) => 2,
            CardBody::CompositeCard(c) => c.cards.len() as u32,
            CardBody::DynamicCall(c) => 1 + c.args.0.len() as u32,
            CardBody::Array(a) => a.len() as u32,
            CardBody::Closure(c) => c.cards.len() as u32,
        }
    }

    pub fn iter_children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut Card> + 'a> {
        match &mut self.body {
            CardBody::Add(b)
            | CardBody::Sub(b)
            | CardBody::Mul(b)
            | CardBody::Div(b)
            | CardBody::Less(b)
            | CardBody::LessOrEq(b)
            | CardBody::Equals(b)
            | CardBody::NotEquals(b)
            | CardBody::And(b)
            | CardBody::Or(b)
            | CardBody::GetProperty(b)
            | CardBody::IfTrue(b)
            | CardBody::IfFalse(b)
            | CardBody::While(b)
            | CardBody::Get(b)
            | CardBody::AppendTable(b)
            | CardBody::Xor(b) => Box::new(b.iter_mut()),
            CardBody::PopTable(u) | CardBody::Len(u) | CardBody::Not(u) | CardBody::Return(u) => {
                Box::new(std::iter::once(u.card.as_mut()))
            }
            CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::Function(_)
            | CardBody::CreateTable
            | CardBody::ReadVar(_)
            | CardBody::NativeFunction(_)
            | CardBody::Abort
            | CardBody::ScalarNil => Box::new(std::iter::empty()),
            CardBody::IfElse(t) | CardBody::SetProperty(t) => Box::new(t.iter_mut()),
            CardBody::CallNative(c) => Box::new(c.args.0.iter_mut()),
            CardBody::Call(c) => Box::new(c.args.0.iter_mut()),
            CardBody::SetGlobalVar(s) | CardBody::SetVar(s) => {
                Box::new(std::iter::once(&mut s.value))
            }
            CardBody::Repeat(r) => Box::new([&mut r.n, &mut r.body].into_iter()),
            CardBody::ForEach(f) => Box::new([f.iterable.as_mut(), f.body.as_mut()].into_iter()),
            CardBody::CompositeCard(c) => Box::new(c.cards.iter_mut()),
            CardBody::DynamicCall(c) => {
                Box::new(std::iter::once(&mut c.function).chain(c.args.0.iter_mut()))
            }
            CardBody::Array(a) => Box::new(a.iter_mut()),
            CardBody::Closure(c) => Box::new(c.cards.iter_mut()),
        }
    }

    pub fn iter_children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Card> + 'a> {
        match &self.body {
            CardBody::Add(b)
            | CardBody::Sub(b)
            | CardBody::Mul(b)
            | CardBody::Div(b)
            | CardBody::Less(b)
            | CardBody::LessOrEq(b)
            | CardBody::Equals(b)
            | CardBody::NotEquals(b)
            | CardBody::And(b)
            | CardBody::Or(b)
            | CardBody::GetProperty(b)
            | CardBody::IfTrue(b)
            | CardBody::IfFalse(b)
            | CardBody::While(b)
            | CardBody::Get(b)
            | CardBody::AppendTable(b)
            | CardBody::Xor(b) => Box::new(b.iter()),
            CardBody::PopTable(u) | CardBody::Len(u) | CardBody::Not(u) | CardBody::Return(u) => {
                Box::new(std::iter::once(u.card.as_ref()))
            }
            CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::Function(_)
            | CardBody::CreateTable
            | CardBody::ReadVar(_)
            | CardBody::NativeFunction(_)
            | CardBody::Abort
            | CardBody::ScalarNil => Box::new(std::iter::empty()),
            CardBody::IfElse(t) | CardBody::SetProperty(t) => Box::new(t.iter()),
            CardBody::CallNative(c) => Box::new(c.args.0.iter()),
            CardBody::Call(c) => Box::new(c.args.0.iter()),
            CardBody::SetGlobalVar(s) | CardBody::SetVar(s) => Box::new(std::iter::once(&s.value)),
            CardBody::Repeat(r) => Box::new([&r.n, &r.body].into_iter()),
            CardBody::ForEach(f) => Box::new([f.iterable.as_ref(), f.body.as_ref()].into_iter()),
            CardBody::CompositeCard(c) => Box::new(c.cards.iter()),
            CardBody::DynamicCall(c) => {
                Box::new(std::iter::once(&c.function).chain(c.args.0.iter()))
            }
            CardBody::Array(a) => Box::new(a.iter()),
            CardBody::Closure(c) => Box::new(c.cards.iter()),
        }
    }

    pub fn get_child_mut(&mut self, i: usize) -> Option<&mut Card> {
        let res;
        match &mut self.body {
            CardBody::CompositeCard(c) => res = c.cards.get_mut(i)?,
            CardBody::Closure(c) => res = c.cards.get_mut(i)?,
            CardBody::Repeat(rep) => match i {
                0 => res = &mut rep.n,
                1 => res = &mut rep.body,
                _ => return None,
            },
            CardBody::IfTrue(c) | CardBody::IfFalse(c) => return c.get_mut(i),
            CardBody::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
                match i {
                    0 => res = a.as_mut(),
                    1 => res = b.as_mut(),
                    _ => return None,
                }
            }
            CardBody::IfElse(children) => return children.get_mut(i),

            CardBody::Add(expr)
            | CardBody::While(expr)
            | CardBody::Sub(expr)
            | CardBody::Mul(expr)
            | CardBody::Div(expr)
            | CardBody::Less(expr)
            | CardBody::LessOrEq(expr)
            | CardBody::Equals(expr)
            | CardBody::NotEquals(expr)
            | CardBody::And(expr)
            | CardBody::Or(expr)
            | CardBody::Xor(expr)
            | CardBody::AppendTable(expr)
            | CardBody::Get(expr)
            | CardBody::GetProperty(expr) => return expr.get_mut(i),
            CardBody::SetProperty(expr) => return expr.get_mut(i),

            CardBody::PopTable(expr)
            | CardBody::Not(expr)
            | CardBody::Return(expr)
            | CardBody::Len(expr) => match i {
                0 => res = &mut expr.card,
                _ => return None,
            },

            CardBody::SetGlobalVar(s) | CardBody::SetVar(s) => match i {
                0 => res = &mut s.value,
                _ => return None,
            },
            CardBody::CallNative(j) => return j.args.0.get_mut(i),
            CardBody::Call(j) => return j.args.0.get_mut(i),
            CardBody::DynamicCall(j) => {
                return (i == 0)
                    .then_some(&mut j.function)
                    .or_else(|| j.args.0.get_mut(i - 1))
            }
            CardBody::Array(cards) => return cards.get_mut(i),
            CardBody::Function(_)
            | CardBody::NativeFunction(_)
            | CardBody::ReadVar(_)
            | CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::ScalarNil
            | CardBody::CreateTable
            | CardBody::Abort => return None,
        }
        Some(res)
    }

    pub fn get_child(&self, i: usize) -> Option<&Card> {
        let res;
        match &self.body {
            CardBody::CompositeCard(c) => res = c.cards.get(i)?,
            CardBody::Closure(c) => res = c.cards.get(i)?,
            CardBody::Repeat(rep) => match i {
                0 => res = &rep.n,
                1 => res = &rep.body,
                _ => return None,
            },
            CardBody::IfTrue(c) | CardBody::IfFalse(c) => return c.get(i),
            CardBody::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_ref();
                match i {
                    0 => res = a.as_ref(),
                    1 => res = b.as_ref(),
                    _ => return None,
                }
            }
            CardBody::IfElse(children) => return children.get(i),
            CardBody::While(expr)
            | CardBody::Add(expr)
            | CardBody::Sub(expr)
            | CardBody::Mul(expr)
            | CardBody::Div(expr)
            | CardBody::Less(expr)
            | CardBody::LessOrEq(expr)
            | CardBody::Equals(expr)
            | CardBody::NotEquals(expr)
            | CardBody::And(expr)
            | CardBody::Or(expr)
            | CardBody::Xor(expr)
            | CardBody::AppendTable(expr)
            | CardBody::Get(expr)
            | CardBody::GetProperty(expr) => return expr.get(i),
            CardBody::SetProperty(expr) => return expr.get(i),

            CardBody::PopTable(expr)
            | CardBody::Not(expr)
            | CardBody::Return(expr)
            | CardBody::Len(expr) => match i {
                0 => res = &expr.card,
                _ => return None,
            },

            CardBody::SetGlobalVar(s) | CardBody::SetVar(s) => match i {
                0 => res = &s.value,
                _ => return None,
            },
            CardBody::CallNative(j) => return j.args.0.get(i),
            CardBody::Call(j) => return j.args.0.get(i),
            CardBody::DynamicCall(j) => {
                return (i == 0)
                    .then_some(&j.function)
                    .or_else(|| j.args.0.get(i - 1))
            }
            CardBody::Array(cards) => return cards.get(i),
            CardBody::Function(_)
            | CardBody::NativeFunction(_)
            | CardBody::ReadVar(_)
            | CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::ScalarNil
            | CardBody::CreateTable
            | CardBody::Abort => return None,
        }
        Some(res)
    }

    pub fn remove_child(&mut self, i: usize) -> Option<Card> {
        let res;
        match &mut self.body {
            CardBody::CompositeCard(c) => {
                if c.cards.len() <= i {
                    return None;
                }
                res = c.cards.remove(i);
            }
            CardBody::Closure(c) => {
                if c.cards.len() <= i {
                    return None;
                }
                res = c.cards.remove(i);
            }
            CardBody::Repeat(rep) => match i {
                0 => res = std::mem::replace(&mut rep.n, CardBody::ScalarInt(0).into()),
                1 => res = std::mem::replace(&mut rep.body, CardBody::ScalarNil.into()),
                _ => return None,
            },
            CardBody::IfTrue(_) | CardBody::IfFalse(_) => {
                let c = self.get_child_mut(i)?;
                res = std::mem::replace::<Card>(c, CardBody::ScalarNil.into());
            }

            CardBody::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
                match i {
                    0 => res = std::mem::replace::<Card>(a.as_mut(), CardBody::ScalarNil.into()),
                    1 => res = std::mem::replace::<Card>(b.as_mut(), CardBody::ScalarNil.into()),
                    _ => return None,
                }
            }
            CardBody::IfElse(children) => {
                let c = children.get_mut(i)?;
                res = std::mem::replace(c, CardBody::ScalarNil.into());
            }
            CardBody::While(_)
            | CardBody::Add(_)
            | CardBody::Sub(_)
            | CardBody::Mul(_)
            | CardBody::Div(_)
            | CardBody::Less(_)
            | CardBody::LessOrEq(_)
            | CardBody::Equals(_)
            | CardBody::NotEquals(_)
            | CardBody::And(_)
            | CardBody::Or(_)
            | CardBody::Xor(_)
            | CardBody::AppendTable(_)
            | CardBody::Get(_)
            | CardBody::SetProperty(_)
            | CardBody::PopTable(_)
            | CardBody::Not(_)
            | CardBody::Return(_)
            | CardBody::Len(_)
            | CardBody::SetGlobalVar(_)
            | CardBody::SetVar(_)
            | CardBody::GetProperty(_) => {
                let c = self.get_child_mut(i)?;
                res = std::mem::replace(c, CardBody::ScalarNil.into());
            }

            CardBody::CallNative(j) => return (i < j.args.0.len()).then(|| j.args.0.remove(i)),
            CardBody::Call(j) => return (i < j.args.0.len()).then(|| j.args.0.remove(i)),
            CardBody::DynamicCall(j) => {
                if i == 0 {
                    res = std::mem::replace(&mut j.function, CardBody::ScalarNil.into());
                } else if i - 1 < j.args.0.len() {
                    res = j.args.0.remove(i - 1);
                } else {
                    return None;
                }
            }
            CardBody::Array(cards) => return (i < cards.len()).then(|| cards.remove(i)),
            CardBody::Function(_)
            | CardBody::NativeFunction(_)
            | CardBody::ReadVar(_)
            | CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::ScalarNil
            | CardBody::CreateTable
            | CardBody::Abort => return None,
        }
        Some(res)
    }

    /// insert a child at the specified index, if the Card is a list, or replace the child at the
    /// index if not
    ///
    /// returns the inserted card on failure
    pub fn insert_child(&mut self, i: usize, card: impl Into<Self>) -> Result<(), Self> {
        let card = card.into();
        match &mut self.body {
            CardBody::CompositeCard(c) => {
                if c.cards.len() < i {
                    return Err(card);
                }
                c.cards.insert(i, card);
            }
            CardBody::Closure(c) => {
                if c.cards.len() < i {
                    return Err(card);
                }
                c.cards.insert(i, card);
            }

            CardBody::ForEach(fe) => {
                let ForEach {
                    i: _,
                    k: _,
                    v: _,
                    iterable: a,
                    body: b,
                } = fe.as_mut();
                match i {
                    0 => *a.as_mut() = card,
                    1 => *b.as_mut() = card,
                    _ => return Err(card),
                };
            }
            CardBody::IfElse(children) => match children.get_mut(i) {
                Some(c) => {
                    *c = card;
                }
                None => return Err(card),
            },
            CardBody::While(_)
            | CardBody::IfTrue(_)
            | CardBody::IfFalse(_)
            | CardBody::Add(_)
            | CardBody::Sub(_)
            | CardBody::Mul(_)
            | CardBody::Div(_)
            | CardBody::Less(_)
            | CardBody::LessOrEq(_)
            | CardBody::Equals(_)
            | CardBody::NotEquals(_)
            | CardBody::And(_)
            | CardBody::Or(_)
            | CardBody::Xor(_)
            | CardBody::AppendTable(_)
            | CardBody::Get(_)
            | CardBody::SetProperty(_)
            | CardBody::PopTable(_)
            | CardBody::Not(_)
            | CardBody::Return(_)
            | CardBody::Len(_)
            | CardBody::SetGlobalVar(_)
            | CardBody::SetVar(_)
            | CardBody::Repeat(_)
            | CardBody::GetProperty(_) => match self.get_child_mut(i) {
                Some(c) => *c = card,
                None => return Err(card),
            },
            CardBody::CallNative(j) => {
                (i <= j.args.0.len()).then(|| j.args.0.insert(i, card));
            }
            CardBody::Call(j) => {
                (i <= j.args.0.len()).then(|| j.args.0.insert(i, card));
            }
            CardBody::DynamicCall(j) => {
                if i == 0 {
                    j.function = card;
                } else if i - 1 <= j.args.0.len() {
                    j.args.0.insert(i - 1, card);
                } else {
                    return Err(card);
                }
            }

            CardBody::Array(children) => {
                if i <= children.len() {
                    children.insert(i, card);
                } else {
                    return Err(card);
                }
            }
            CardBody::Function(_)
            | CardBody::NativeFunction(_)
            | CardBody::ReadVar(_)
            | CardBody::ScalarInt(_)
            | CardBody::ScalarFloat(_)
            | CardBody::StringLiteral(_)
            | CardBody::Comment(_)
            | CardBody::ScalarNil
            | CardBody::CreateTable
            | CardBody::Abort => return Err(card),
        }
        Ok(())
    }

    /// Return Ok(old card) on success, return the input card in fail
    pub fn replace_child(&mut self, i: usize, card: impl Into<Self>) -> Result<Self, Self> {
        let card = card.into();
        match self.get_child_mut(i) {
            Some(c) => Ok(std::mem::replace(c, card)),
            None => Err(card),
        }
    }

    pub fn return_card(c: impl Into<Self>) -> Self {
        CardBody::Return(UnaryExpression {
            card: Box::new(c.into()),
        })
        .into()
    }

    pub fn set_property(
        value: impl Into<Self>,
        table: impl Into<Self>,
        key: impl Into<Self>,
    ) -> Self {
        CardBody::SetProperty(Box::new([value.into(), table.into(), key.into()])).into()
    }

    pub fn get_property(table: impl Into<Self>, key: impl Into<Self>) -> Self {
        CardBody::GetProperty(Box::new([table.into(), key.into()])).into()
    }

    pub fn dynamic_call(function: impl Into<Card>, args: impl Into<Arguments>) -> Self {
        CardBody::DynamicCall(Box::new(DynamicJump {
            args: args.into(),
            function: function.into(),
        }))
        .into()
    }
}

#[derive(Debug, Clone, Default)]
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
        CardBody::CompositeCard(Box::new(value)).into()
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
    pub fn new(c: impl Into<Card>) -> Self {
        Self {
            card: Box::new(c.into()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_indexing_consistent() {
        let card: Card = CardBody::ForEach(Box::new(ForEach {
            i: None,
            k: None,
            v: None,
            iterable: Box::new(CardBody::ScalarInt(42).into()),
            body: Box::new(Card::string_card("winnie")),
        }))
        .into();

        for (i, a) in card.iter_children().enumerate() {
            let _b = card.get_child(i);
            assert!(matches!(Some(a), _b));
        }
    }
}
