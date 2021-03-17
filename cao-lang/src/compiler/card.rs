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
    ExitWithCode(IntegerNode),
    ScalarInt(IntegerNode),
    ScalarNull,
    ScalarFloat(FloatNode),
    ScalarLabel(IntegerNode),
    ScalarArray(IntegerNode),
    StringLiteral(StringNode),
    Call(CallNode),
    JumpIfTrue(JumpToLane),
    JumpIfFalse(JumpToLane),
    Jump(JumpToLane),
    SetGlobalVar(VarNode),
    ReadGlobalVar(VarNode),
    Return,
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
            Card::Less => "Less",
            Card::LessOrEq => "LessOrEq",
            Card::Equals => "Equals",
            Card::NotEquals => "NotEquals",
            Card::Pop => "Pop",
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
        }
    }

    /// Translate this Card into an Instruction.
    pub fn instruction(&self) -> Option<Instruction> {
        match self {
            Card::ExitWithCode(_) => None,

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

    // Trigger compilation errors for newly added instructions so we don't forget implementing them
    // here
    #[allow(unused)]
    fn _instruction_to_node(instr: Instruction) {
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
            | Instruction::ScalarInt
            | Instruction::Add
            | Instruction::ScalarNull
            | Instruction::Return
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpToLane(pub String);

#[derive(Debug, Clone, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VarNode(pub VarName);
