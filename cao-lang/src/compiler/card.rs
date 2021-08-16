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
#[cfg_attr(feature = "serde", serde(tag = "ty", content = "val"))]
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
    IfTrue(LaneNode),
    IfFalse(LaneNode),
    IfElse { then: LaneNode, r#else: LaneNode },
    Jump(LaneNode),
    SetGlobalVar(VarNode),
    SetVar(VarNode),
    ReadVar(VarNode),
    Repeat(LaneNode),
    While(LaneNode),
    ForEach { variable: VarNode, lane: LaneNode },
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            // Card::GetByKey => "GetByKey",
            // Card::SetByKey => "SetByKey",
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
            | Card::ForEach { .. } => None,

            Card::GetProperty => Some(Instruction::GetProperty),
            Card::SetProperty => Some(Instruction::SetProperty),
            Card::CreateTable => Some(Instruction::InitTable),
            Card::And => Some(Instruction::And),
            Card::Abort => Some(Instruction::Exit),
            Card::Not => Some(Instruction::Not),
            Card::Or => Some(Instruction::Or),
            Card::Xor => Some(Instruction::Xor),
            Card::Pass => Some(Instruction::Pass),
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
            | Instruction::BeginForEach
            | Instruction::Pass => {}
        };
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LaneNode {
    LaneName(String),
    LaneId(usize),
}

impl Display for LaneNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LaneNode::LaneName(n) => write!(f, "{}", n),
            LaneNode::LaneId(n) => write!(f, "#{}", n),
        }
    }
}

impl Default for LaneNode {
    fn default() -> Self {
        Self::LaneId(0)
    }
}

impl From<LaneNode> for Handle {
    fn from(ln: LaneNode) -> Self {
        Self::from(&ln)
    }
}

impl<'a> From<&'a LaneNode> for Handle {
    fn from(ln: &'a LaneNode) -> Self {
        match ln {
            LaneNode::LaneName(s) => Handle::from_str(s.as_str()).unwrap(),
            LaneNode::LaneId(i) => Handle::from_i64(*i as i64),
        }
    }
}
