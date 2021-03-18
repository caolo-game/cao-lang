use super::Card;
use crate::scalar::Scalar;
use crate::traits::ByteEncodeble;
use crate::Pointer;
use crate::VarName;
use crate::{subprogram_description, SubProgram, SubProgramType};

pub fn get_instruction_descriptions() -> Vec<SubProgram<'static>> {
    vec![
        get_desc(Card::Pass),
        get_desc(Card::Add),
        get_desc(Card::Sub),
        get_desc(Card::Mul),
        get_desc(Card::Div),
        get_desc(Card::CopyLast),
        get_desc(Card::Less),
        get_desc(Card::LessOrEq),
        get_desc(Card::Equals),
        get_desc(Card::NotEquals),
        get_desc(Card::Pop),
        get_desc(Card::ClearStack),
        get_desc(Card::And),
        get_desc(Card::Or),
        get_desc(Card::Xor),
        get_desc(Card::ScalarInt(Default::default())),
        get_desc(Card::ScalarFloat(Default::default())),
        get_desc(Card::ScalarArray(Default::default())),
        get_desc(Card::StringLiteral(Default::default())),
        get_desc(Card::JumpIfTrue(Default::default())),
        get_desc(Card::JumpIfFalse(Default::default())),
        get_desc(Card::Jump(Default::default())),
        get_desc(Card::SetGlobalVar(Default::default())),
        get_desc(Card::ReadGlobalVar(Default::default())),
        get_desc(Card::ExitWithCode(Default::default())),
        get_desc(Card::ScalarNull),
        get_desc(Card::Return),
        get_desc(Card::Repeat(Default::default())),
    ]
}

#[inline(always)]
fn get_desc(node: Card) -> SubProgram<'static> {
    match node {
        Card::Call(_) | Card::ScalarLabel(_) | Card::Exit => unreachable!(),
        Card::Pass => subprogram_description!(
            "Pass",
            "Do nothing",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        Card::And => subprogram_description!(
            "And",
            "Logical And",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        Card::Or => subprogram_description!(
            "Or",
            "Logical inclusive Or",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        Card::Xor => subprogram_description!(
            "Xor",
            "Logical exclusive Or",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        Card::Add => subprogram_description!(
            "Add",
            "Add two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::Sub => subprogram_description!(
            "Sub",
            "Subtract two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::Mul => subprogram_description!(
            "Mul",
            "Multiply two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::Div => subprogram_description!(
            "Div",
            "Divide two scalars",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::CopyLast => subprogram_description!(
            "CopyLast",
            "Duplicate the last item on the stack",
            SubProgramType::Instruction,
            [Scalar],
            [Scalar, Scalar],
            []
        ),
        Card::Less => subprogram_description!(
            "Less",
            "Return 1 if the first input is less than the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::LessOrEq => subprogram_description!(
            "LessOrEq",
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::Equals => subprogram_description!(
            "Equals",
            "Return 1 if the inputs are equal, 0 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::ScalarNull => subprogram_description!(
            "ScalarNull",
            "Push a `null` scalar onto the stack",
            SubProgramType::Instruction,
            [],
            [Scalar],
            []
        ),

        Card::NotEquals => subprogram_description!(
            "NotEquals",
            "Return 0 if the inputs are equal, 1 otherwise",
            SubProgramType::Instruction,
            [Scalar, Scalar],
            [Scalar],
            []
        ),

        Card::Pop => subprogram_description!(
            "Pop",
            "Pops the top elements on the stack and discards it",
            SubProgramType::Instruction,
            [Scalar],
            [],
            []
        ),

        Card::ClearStack => subprogram_description!(
            "ClearStack",
            "Clears the stack",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),

        Card::ExitWithCode(_) => subprogram_description!(
            "ExitWithCode",
            "Exit the program returning the provided status code",
            SubProgramType::Instruction,
            [],
            [],
            [i32]
        ),

        Card::ScalarInt(_) => subprogram_description!(
            "ScalarInt",
            "Make an integer",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [i32]
        ),

        Card::ScalarFloat(_) => subprogram_description!(
            "ScalarFloat",
            "Make a real number",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [f32]
        ),

        Card::ScalarArray(_) => subprogram_description!(
            "ScalarArray",
            "Make an array by providing a number and values",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [i32]
        ),

        Card::StringLiteral(_) => subprogram_description!(
            "StringLiteral",
            "Make a text",
            SubProgramType::Instruction,
            [],
            [Scalar],
            [String]
        ),

        Card::JumpIfTrue(_) => subprogram_description!(
            "JumpIfTrue",
            "Jump to the input node if the last value is true else do nothing.",
            SubProgramType::Branch,
            [Scalar],
            [],
            [String]
        ),

        Card::JumpIfFalse(_) => subprogram_description!(
            "JumpIfFalse",
            "Jump to the input node if the last value is false else do nothing.",
            SubProgramType::Branch,
            [Scalar],
            [],
            [String]
        ),

        Card::Jump(_) => subprogram_description!(
            "Jump",
            "Jump to the input node.",
            SubProgramType::Branch,
            [],
            [],
            [String]
        ),

        Card::SetGlobalVar(_) => subprogram_description!(
            "SetVar",
            "Sets the value of a variable",
            SubProgramType::Instruction,
            [Pointer],
            [],
            [VarName]
        ),

        Card::ReadGlobalVar(_) => subprogram_description!(
            "ReadVar",
            "Read the value of a variable",
            SubProgramType::Instruction,
            [],
            [Pointer],
            [VarName]
        ),

        Card::Return => subprogram_description!(
            "Return",
            "Return to where this Lane was called",
            SubProgramType::Branch,
            [],
            [],
            []
        ),

        Card::Repeat(_) => subprogram_description!(
            "Repeat",
            "Repeat a lane the input number of times",
            SubProgramType::Branch,
            [Scalar],
            [],
            [String]
        ),
    }
}
