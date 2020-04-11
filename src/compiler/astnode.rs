use super::*;
use crate::InputString;
use crate::VarName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub node: InstructionNode,
    pub child: Option<NodeId>,
}

impl Default for AstNode {
    fn default() -> Self {
        Self {
            node: InstructionNode::Pass,
            child: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionNode {
    Start,
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
    ScalarInt(IntegerNode),
    ScalarFloat(FloatNode),
    ScalarLabel(IntegerNode),
    ScalarArray(IntegerNode),
    StringLiteral(StringNode),
    Call(CallNode),
    JumpIfTrue(JumpNode),
    Jump(JumpNode),
    SetVar(VarNode),
    ReadVar(VarNode),
    SubProgram(SubProgramNode),
}

impl InstructionNode {
    /// Translate this Node into an Instruction.
    /// Returns None for non-assemly nodes.
    pub fn instruction(&self) -> Option<Instruction> {
        match self {
            InstructionNode::Start => Some(Instruction::Start),
            InstructionNode::Pass => Some(Instruction::Pass),
            InstructionNode::Add => Some(Instruction::Add),
            InstructionNode::Sub => Some(Instruction::Sub),
            InstructionNode::Mul => Some(Instruction::Mul),
            InstructionNode::Div => Some(Instruction::Div),
            InstructionNode::Exit => Some(Instruction::Exit),
            InstructionNode::CopyLast => Some(Instruction::CopyLast),
            InstructionNode::Less => Some(Instruction::Less),
            InstructionNode::LessOrEq => Some(Instruction::LessOrEq),
            InstructionNode::Equals => Some(Instruction::Equals),
            InstructionNode::NotEquals => Some(Instruction::NotEquals),
            InstructionNode::Pop => Some(Instruction::Pop),
            InstructionNode::ScalarInt(_) => Some(Instruction::ScalarInt),
            InstructionNode::ScalarFloat(_) => Some(Instruction::ScalarFloat),
            InstructionNode::ScalarArray(_) => Some(Instruction::ScalarArray),
            InstructionNode::ScalarLabel(_) => Some(Instruction::ScalarLabel),
            InstructionNode::Call(_) => Some(Instruction::Call),
            InstructionNode::JumpIfTrue(_) => Some(Instruction::JumpIfTrue),
            InstructionNode::Jump(_) => Some(Instruction::Jump),
            InstructionNode::StringLiteral(_) => Some(Instruction::StringLiteral),
            InstructionNode::SetVar(_) => Some(Instruction::SetVar),
            InstructionNode::ReadVar(_) => Some(Instruction::ReadVar),
            InstructionNode::SubProgram(_) => None,
        }
    }

    // Trigger compilation errors for newly added instructions so we don't forget implementing them
    // here
    #[allow(unused)]
    fn _instruction_to_node(instr: Instruction) {
        match instr {
            Instruction::SetVar
            | Instruction::ReadVar
            | Instruction::Pop
            | Instruction::Less
            | Instruction::LessOrEq
            | Instruction::Equals
            | Instruction::NotEquals
            | Instruction::Exit
            | Instruction::StringLiteral
            | Instruction::Start
            | Instruction::JumpIfTrue
            | Instruction::Jump
            | Instruction::CopyLast
            | Instruction::Call
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::ScalarArray
            | Instruction::ScalarLabel
            | Instruction::ScalarFloat
            | Instruction::ScalarInt
            | Instruction::Add
            | Instruction::Pass => {}
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct IntegerNode {
    pub value: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct FloatNode {
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CallNode {
    pub function: InputString,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubProgramNode {
    pub name: InputString,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StringNode {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct JumpNode {
    pub nodeid: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct VarNode {
    pub name: VarName,
}
