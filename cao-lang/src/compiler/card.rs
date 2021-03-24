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
    Exit,
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
    ScalarNull,
    ScalarInt(IntegerNode),
    ScalarFloat(FloatNode),
    ScalarLabel(IntegerNode),
    ScalarArray(IntegerNode),
    ExitWithCode(IntegerNode),
    StringLiteral(StringNode),
    Call(CallNode),
    JumpIfTrue(LaneNode),
    JumpIfFalse(LaneNode),
    Jump(LaneNode),
    SetGlobalVar(VarNode),
    ReadGlobalVar(VarNode),
    Repeat(LaneNode),
    While(LaneNode),
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            Card::Pass => "Pass",
            Card::Add => "Add",
            Card::Sub => "Sub",
            Card::Mul => "Mul",
            Card::Div => "Div",
            Card::Exit => "Exit",
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
            Card::ExitWithCode(_) => "ExitWithCode",
            Card::ScalarInt(_) => "ScalarInt",
            Card::ScalarFloat(_) => "ScalarFloat",
            Card::ScalarLabel(_) => "ScalarLabel",
            Card::ScalarArray(_) => "ScalarArray",
            Card::StringLiteral(_) => "StringLiteral",
            Card::Call(_) => "Call",
            Card::JumpIfTrue(_) => "JumpIfTrue",
            Card::JumpIfFalse(_) => "JumpIfFalse",
            Card::Jump(_) => "Jump",
            Card::SetGlobalVar(_) => "SetGlobalVar",
            Card::ReadGlobalVar(_) => "ReadGlobalVar",
            Card::ClearStack => "ClearStack",
            Card::ScalarNull => "ScalarNull",
            Card::Return => "Return",
            Card::Repeat(_) => "Repeat",
            Card::While(_) => "While",
        }
    }

    /// Translate this Card into an Instruction, if possible.
    /// Some cards expand to multiple instructions, these are handled separately
    pub fn instruction(&self) -> Option<Instruction> {
        match self {
            Card::While(_) | Card::Repeat(_) | Card::ExitWithCode(_) => None,

            Card::And => Some(Instruction::And),
            Card::Not => Some(Instruction::Not),
            Card::Or => Some(Instruction::Or),
            Card::Xor => Some(Instruction::Xor),
            Card::Pass => Some(Instruction::Pass),
            Card::Add => Some(Instruction::Add),
            Card::Sub => Some(Instruction::Sub),
            Card::Mul => Some(Instruction::Mul),
            Card::Div => Some(Instruction::Div),
            Card::Exit => Some(Instruction::Exit),
            Card::CopyLast => Some(Instruction::CopyLast),
            Card::Less => Some(Instruction::Less),
            Card::LessOrEq => Some(Instruction::LessOrEq),
            Card::Equals => Some(Instruction::Equals),
            Card::NotEquals => Some(Instruction::NotEquals),
            Card::Pop => Some(Instruction::Pop),
            Card::ScalarInt(_) => Some(Instruction::ScalarInt),
            Card::ScalarFloat(_) => Some(Instruction::ScalarFloat),
            Card::ScalarArray(_) => Some(Instruction::ScalarArray),
            Card::ScalarLabel(_) => Some(Instruction::ScalarLabel),
            Card::Call(_) => Some(Instruction::Call),
            Card::JumpIfTrue(_) => Some(Instruction::JumpIfTrue),
            Card::JumpIfFalse(_) => Some(Instruction::JumpIfFalse),
            Card::Jump(_) => Some(Instruction::Jump),
            Card::StringLiteral(_) => Some(Instruction::StringLiteral),
            Card::SetGlobalVar(_) => Some(Instruction::SetGlobalVar),
            Card::ReadGlobalVar(_) => Some(Instruction::ReadGlobalVar),
            Card::ClearStack => Some(Instruction::ClearStack),
            Card::ScalarNull => Some(Instruction::ScalarNull),
            Card::Return => Some(Instruction::Return),
        }
    }

    // Trigger compilation errors for newly added instructions,
    // so we don't forget implementing a card for them
    #[allow(unused)]
    fn __instruction_to_node(instr: Instruction) {
        match instr {
            Instruction::SetGlobalVar
            | Instruction::Breadcrumb
            | Instruction::ReadGlobalVar
            | Instruction::Pop
            | Instruction::Less
            | Instruction::LessOrEq
            | Instruction::Equals
            | Instruction::NotEquals
            | Instruction::Exit
            | Instruction::StringLiteral
            | Instruction::JumpIfTrue
            | Instruction::JumpIfFalse
            | Instruction::Jump
            | Instruction::CopyLast
            | Instruction::Call
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::ScalarArray
            | Instruction::ScalarLabel
            | Instruction::ClearStack
            | Instruction::ScalarFloat
            | Instruction::And
            | Instruction::Not
            | Instruction::Or
            | Instruction::Xor
            | Instruction::ScalarInt
            | Instruction::Add
            | Instruction::ScalarNull
            | Instruction::ScopeStart
            | Instruction::ScopeEnd
            | Instruction::Return
            | Instruction::Remember
            | Instruction::SwapLast
            | Instruction::Goto
            | Instruction::GotoIfTrue
            | Instruction::Pass => {}
        };
    }
}

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntegerNode(pub i32);

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FloatNode(pub f32);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode(pub InputString);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubProgramNode(pub InputString);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringNode(pub String);

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VarNode(pub VarName);

impl VarNode {
    /// panics if the string is too long
    pub fn from_str_unchecked(s: &str) -> Self {
        Self(VarName::from(s).expect("Failed to parse variable name"))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LaneNode {
    LaneName(String),
    LaneId(usize),
}

impl Default for LaneNode {
    fn default() -> Self {
        Self::LaneId(0)
    }
}
