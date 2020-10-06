use crate::scalar::Scalar;
use crate::traits::ByteEncodeble;
use crate::InputString;
use crate::NodeId;
use crate::TPointer;
use crate::VarName;
use crate::{subprogram_description, SubProgram};

pub fn get_instruction_descriptions() -> Vec<SubProgram<'static>> {
    vec![
        subprogram_description!(Add, "Add two scalars", [Scalar, Scalar], [Scalar], []),
        subprogram_description!(Sub, "Subtract two scalars", [Scalar, Scalar], [Scalar], []),
        subprogram_description!(Mul, "Multiply two scalars", [Scalar, Scalar], [Scalar], []),
        subprogram_description!(Div, "Divide two scalars", [Scalar, Scalar], [Scalar], []),
        subprogram_description!(Start, "Start of the program", [], [], []),
        subprogram_description!(Pass, "Do nothing", [], [], []),
        subprogram_description!(ScalarInt, "Make an integer", [], [Scalar], [i32]),
        subprogram_description!(ScalarFloat, "Make a real number", [], [Scalar], [f32]),
        subprogram_description!(StringLiteral, "Make a text", [], [Scalar], [String]),
        subprogram_description!(
            JumpIfTrue,
            "Jump to the input node if the last value is true else do nothing.",
            [Scalar],
            [],
            [NodeId]
        ),
        subprogram_description!(
            Equals,
            "Return 1 if the inputs are equal, 0 otherwise",
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            NotEquals,
            "Return 0 if the inputs are equal, 1 otherwise",
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            Less,
            "Return 1 if the first input is less than the second, 0 otherwise",
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            LessOrEq,
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            [Scalar, Scalar],
            [Scalar],
            []
        ),
        subprogram_description!(
            Pop,
            "Pops the top elements on the stack and discards it",
            [Scalar],
            [],
            []
        ),
        subprogram_description!(
            SetVar,
            "Sets the value of a variable",
            [TPointer],
            [],
            [VarName]
        ),
        subprogram_description!(
            ReadVar,
            "Read the value of a variable",
            [],
            [TPointer],
            [VarName]
        ),
        subprogram_description!(
            SubProgram,
            "Call a SubProgram by name",
            [],
            [],
            [InputString]
        ),
    ]
}
