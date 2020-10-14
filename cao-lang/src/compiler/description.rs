use crate::scalar::Scalar;
use crate::traits::ByteEncodeble;
use crate::InputString;
use crate::NodeId;
use crate::TPointer;
use crate::VarName;
use crate::{subprogram_description, SubProgram, SubProgramType};

pub fn get_instruction_descriptions() -> Vec<SubProgram<'static>> {
    vec![
        subprogram_description!(
            "Add",
            "Add two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Sub",
            "Subtract two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Mul",
            "Multiply two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Div",
            "Divide two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Start",
            "Start of the program",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        subprogram_description!(
            "Pass",
            "Do nothing",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        subprogram_description!(
            "ScalarInt",
            "Make an integer",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [i32]
        ),
        subprogram_description!(
            "ScalarFloat",
            "Make a real number",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [f32]
        ),
        subprogram_description!(
            "StringLiteral",
            "Make a text",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [String]
        ),
        subprogram_description!(
            "JumpIfTrue",
            "Jump to the input node if the last value is true else do nothing.",
            SubProgramType::Instruction,
            [Scalar],
            [],
            [NodeId]
        ),
        subprogram_description!(
            "Equals",
            "Return 1 if the inputs are equal, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "NotEquals",
            "Return 0 if the inputs are equal, 1 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Less",
            "Return 1 if the first input is less than the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "LessOrEq",
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            "Pop",
            "Pops the top elements on the stack and discards it",
            SubProgramType::Instruction,
            [Scalar],
            [],
            []
        ),
        subprogram_description!(
            "SetVar",
            "Sets the value of a variable",
            SubProgramType::Instruction,
            [TPointer],
            [],
            [VarName]
        ),
        subprogram_description!(
            "ReadVar",
            "Read the value of a variable",
            SubProgramType::Instruction,
            [],
            [TPointer],
            [VarName]
        ),
        subprogram_description!(
            "SubProgram",
            "Call a SubProgram by name",
            SubProgramType::Undefined,
            [],
            [],
            [InputString]
        ),
    ]
}
