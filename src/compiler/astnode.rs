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
}

impl InstructionNode {
    pub fn instruction(&self) -> Instruction {
        match self {
            InstructionNode::Start => Instruction::Start,
            InstructionNode::Pass => Instruction::Pass,
            InstructionNode::Add => Instruction::Add,
            InstructionNode::Sub => Instruction::Sub,
            InstructionNode::Mul => Instruction::Mul,
            InstructionNode::Div => Instruction::Div,
            InstructionNode::Exit => Instruction::Exit,
            InstructionNode::CopyLast => Instruction::CopyLast,
            InstructionNode::Less => Instruction::Less,
            InstructionNode::LessOrEq => Instruction::LessOrEq,
            InstructionNode::Equals => Instruction::Equals,
            InstructionNode::NotEquals => Instruction::NotEquals,
            InstructionNode::Pop => Instruction::Pop,
            InstructionNode::ScalarInt(_) => Instruction::ScalarInt,
            InstructionNode::ScalarFloat(_) => Instruction::ScalarFloat,
            InstructionNode::ScalarArray(_) => Instruction::ScalarArray,
            InstructionNode::ScalarLabel(_) => Instruction::ScalarLabel,
            InstructionNode::Call(_) => Instruction::Call,
            InstructionNode::JumpIfTrue(_) => Instruction::JumpIfTrue,
            InstructionNode::Jump(_) => Instruction::Jump,
            InstructionNode::StringLiteral(_) => Instruction::StringLiteral,
            InstructionNode::SetVar(_) => Instruction::SetVar,
            InstructionNode::ReadVar(_) => Instruction::ReadVar,
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
