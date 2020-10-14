use super::InstructionNode;
use crate::scalar::Scalar;
use crate::traits::ByteEncodeble;
use crate::InputString;
use crate::NodeId;
use crate::TPointer;
use crate::VarName;
use crate::{subprogram_description, SubProgram, SubProgramType};

pub fn get_instruction_descriptions() -> [SubProgram<'static>; 23] {
    [
        get_desc(InstructionNode::Start),
        get_desc(InstructionNode::Pass),
        get_desc(InstructionNode::Add),
        get_desc(InstructionNode::Sub),
        get_desc(InstructionNode::Mul),
        get_desc(InstructionNode::Div),
        get_desc(InstructionNode::CopyLast),
        get_desc(InstructionNode::Less),
        get_desc(InstructionNode::LessOrEq),
        get_desc(InstructionNode::Equals),
        get_desc(InstructionNode::NotEquals),
        get_desc(InstructionNode::Pop),
        get_desc(InstructionNode::ClearStack),
        get_desc(InstructionNode::ScalarInt(Default::default())),
        get_desc(InstructionNode::ScalarFloat(Default::default())),
        get_desc(InstructionNode::ScalarArray(Default::default())),
        get_desc(InstructionNode::StringLiteral(Default::default())),
        get_desc(InstructionNode::JumpIfTrue(Default::default())),
        get_desc(InstructionNode::JumpIfFalse(Default::default())),
        get_desc(InstructionNode::Jump(Default::default())),
        get_desc(InstructionNode::SetVar(Default::default())),
        get_desc(InstructionNode::ReadVar(Default::default())),
        get_desc(InstructionNode::SubProgram(Default::default())),
    ]
}

#[inline(always)]
fn get_desc(node: InstructionNode) -> SubProgram<'static> {
    match node {
        InstructionNode::Call(_) | InstructionNode::ScalarLabel(_) | InstructionNode::Exit => {
            unreachable!()
        }
        InstructionNode::Start => subprogram_description!(
            "Start",
            "Start of the program",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        InstructionNode::Pass => subprogram_description!(
            "Pass",
            "Do nothing",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        InstructionNode::Add => subprogram_description!(
            "Add",
            "Add two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::Sub => subprogram_description!(
            "Sub",
            "Subtract two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::Mul => subprogram_description!(
            "Mul",
            "Multiply two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::Div => subprogram_description!(
            "Div",
            "Divide two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::CopyLast => subprogram_description!(
            "CopyLast",
            "Duplicate the last item on the stack",
            SubProgramType::Instruction,
            [Scalar],
            [Scalar, Scalar],
            []
        ),
        InstructionNode::Less => subprogram_description!(
            "Less",
            "Return 1 if the first input is less than the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::LessOrEq => subprogram_description!(
            "LessOrEq",
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::Equals => subprogram_description!(
            "Equals",
            "Return 1 if the inputs are equal, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::NotEquals => subprogram_description!(
            "NotEquals",
            "Return 0 if the inputs are equal, 1 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        InstructionNode::Pop => subprogram_description!(
            "Pop",
            "Pops the top elements on the stack and discards it",
            SubProgramType::Instruction,
            [Scalar],
            [],
            []
        ),

        InstructionNode::ClearStack => subprogram_description!(
            "ClearStack",
            "Clears the stack",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),

        InstructionNode::ScalarInt(_) => subprogram_description!(
            "ScalarInt",
            "Make an integer",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [i32]
        ),

        InstructionNode::ScalarFloat(_) => subprogram_description!(
            "ScalarFloat",
            "Make a real number",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [f32]
        ),

        InstructionNode::ScalarArray(_) => subprogram_description!(
            "ScalarArray",
            "Make an array by providing a number and values",
            SubProgramType::Instruction,
            [Scalar],
            [Scalar],
            []
        ),

        InstructionNode::StringLiteral(_) => subprogram_description!(
            "StringLiteral",
            "Make a text",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [String]
        ),

        InstructionNode::JumpIfTrue(_) => subprogram_description!(
            "JumpIfTrue",
            "Jump to the input node if the last value is true else do nothing.",
            SubProgramType::Branch,
            [Scalar],
            [],
            [NodeId]
        ),

        InstructionNode::JumpIfFalse(_) => subprogram_description!(
            "JumpIfFalse",
            "Jump to the input node if the last value is false else do nothing.",
            SubProgramType::Branch,
            [Scalar],
            [],
            [NodeId]
        ),

        InstructionNode::Jump(_) => subprogram_description!(
            "Jump",
            "Jump to the input node.",
            SubProgramType::Branch,
            [],
            [],
            [NodeId]
        ),

        InstructionNode::SetVar(_) => subprogram_description!(
            "SetVar",
            "Sets the value of a variable",
            SubProgramType::Instruction,
            [TPointer],
            [],
            [VarName]
        ),

        InstructionNode::ReadVar(_) => subprogram_description!(
            "ReadVar",
            "Read the value of a variable",
            SubProgramType::Instruction,
            [],
            [TPointer],
            [VarName]
        ),
        InstructionNode::SubProgram(_) => subprogram_description!(
            "SubProgram",
            "Call a SubProgram by name",
            SubProgramType::Undefined,
            [],
            [],
            [InputString]
        ),
    }
}
