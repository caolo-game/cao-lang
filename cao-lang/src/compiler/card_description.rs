use super::Card;
use crate::value::Value;
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
        get_desc(Card::Not),
        get_desc(Card::ScalarInt(Default::default())),
        get_desc(Card::ScalarFloat(Default::default())),
        get_desc(Card::StringLiteral(Default::default())),
        get_desc(Card::IfTrue(Default::default())),
        get_desc(Card::IfFalse(Default::default())),
        get_desc(Card::Jump(Default::default())),
        get_desc(Card::SetGlobalVar(Default::default())),
        get_desc(Card::ReadVar(Default::default())),
        get_desc(Card::Abort),
        get_desc(Card::ScalarNil),
        get_desc(Card::Return),
        get_desc(Card::Repeat(Default::default())),
        get_desc(Card::While(Default::default())),
        get_desc(Card::IfElse {
            then: Default::default(),
            r#else: Default::default(),
        }),
    ]
}

#[inline(always)]
fn get_desc(node: Card) -> SubProgram<'static> {
    match node {
        Card::CallNative(_) | Card::ScalarLabel(_)  => unreachable!(),
        Card::Pass => subprogram_description!(
            "Pass",
            "Do nothing",
            SubProgramType::Instruction,
            [],
            [],
            []
        ),
        Card::Not => subprogram_description!(
            "Not",
            "Logically negates the value on the top of the stack",
            SubProgramType::Instruction,
            [Value],
            [Value],
            []
        ),
        Card::And => subprogram_description!(
            "And",
            "Logical And",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),
        Card::Or => subprogram_description!(
            "Or",
            "Logical inclusive Or",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),
        Card::Xor => subprogram_description!(
            "Xor",
            "Logical exclusive Or",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),
        Card::Add => subprogram_description!(
            "Add",
            "Add two scalars",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::Sub => subprogram_description!(
            "Sub",
            "Subtract two scalars",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::Mul => subprogram_description!(
            "Mul",
            "Multiply two scalars",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::Div => subprogram_description!(
            "Div",
            "Divide two scalars",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::CopyLast => subprogram_description!(
            "CopyLast",
            "Duplicate the last item on the stack",
            SubProgramType::Instruction,
            [Value],
            [Value, Value],
            []
        ),
        Card::Less => subprogram_description!(
            "Less",
            "Return 1 if the first input is less than the second, 0 otherwise",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::LessOrEq => subprogram_description!(
            "LessOrEq",
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::Equals => subprogram_description!(
            "Equals",
            "Return 1 if the inputs are equal, 0 otherwise",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::ScalarNil => subprogram_description!(
            "ScalarNil",
            "Push a `Nil` value onto the stack",
            SubProgramType::Instruction,
            [],
            [Value],
            []
        ),

        Card::NotEquals => subprogram_description!(
            "NotEquals",
            "Return 0 if the inputs are equal, 1 otherwise",
            SubProgramType::Instruction,
            [Value, Value],
            [Value],
            []
        ),

        Card::Pop => subprogram_description!(
            "Pop",
            "Pops the top elements on the stack and discards it",
            SubProgramType::Instruction,
            [Value],
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

        Card::Abort => subprogram_description!(
            "Abort",
            "Exit the program",
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
            [Value],
            [i32]
        ),

        Card::ScalarFloat(_) => subprogram_description!(
            "ScalarFloat",
            "Make a real number",
            SubProgramType::Instruction,
            [],
            [Value],
            [f32]
        ),

        Card::StringLiteral(_) => subprogram_description!(
            "StringLiteral",
            "Make a text",
            SubProgramType::Instruction,
            [],
            [Value],
            [str]
        ),

        Card::IfTrue(_) => subprogram_description!(
            "IfTrue",
            "Jump to the input lane if the last value is true else do nothing.",
            SubProgramType::Branch,
            [Value],
            [],
            [str]
        ),

        Card::IfFalse(_) => subprogram_description!(
            "IfFalse",
            "Jump to the input lane if the last value is false else do nothing.",
            SubProgramType::Branch,
            [Value],
            [],
            [str]
        ),

        Card::IfElse { .. } => subprogram_description!(
            "IfElse",
            "Jump to the input lane if the last value is true else jump to the second input lane.",
            SubProgramType::Branch,
            [Value],
            [],
            [str, str]
        ),

        Card::Jump(_) => subprogram_description!(
            "Jump",
            "Jump to the input lane.",
            SubProgramType::Branch,
            [],
            [],
            [str]
        ),

        Card::SetGlobalVar(_) => subprogram_description!(
            "SetGlobalVar",
            "Sets the value of a global variable",
            SubProgramType::Instruction,
            [Pointer],
            [],
            [VarName]
        ),
        Card::SetLocalVar(_) => subprogram_description!(
            "SetLocalVar",
            "Sets the value of a local variable. Local variables are only usable in the Lane they were created in.",
            SubProgramType::Instruction,
            [Pointer],
            [],
            [VarName]
        ),

        Card::ReadVar(_) => subprogram_description!(
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
            [Value],
            [],
            [str]
        ),

        Card::While(_) => subprogram_description!(
            "While",
            "Repeat a lane until the lane's last value is 0",
            SubProgramType::Branch,
            [],
            [],
            [str]
        ),
    }
}
