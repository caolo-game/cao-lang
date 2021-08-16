use super::Card;
use crate::{subprogram_description, SubProgram, SubProgramType};

#[derive(Debug, Clone, Copy)]
pub enum PropertyName {
    Integer,
    Float,
    Number,
    Value,
    Text,
    Variable,
    Object,
    Boolean,
}

impl PropertyName {
    /// return a list of all available properties
    pub fn all_props() -> &'static [PropertyName] {
        &[
            PropertyName::Integer,
            PropertyName::Float,
            PropertyName::Number,
            PropertyName::Value,
            PropertyName::Text,
            PropertyName::Variable,
            PropertyName::Object,
            PropertyName::Boolean,
        ]
    }

    pub fn to_str(self) -> &'static str {
        match self {
            PropertyName::Integer => "Integer",
            PropertyName::Float => "Float",
            PropertyName::Number => "Number",
            PropertyName::Value => "Value",
            PropertyName::Text => "Text",
            PropertyName::Variable => "Variable",
            PropertyName::Object => "Object",
            PropertyName::Boolean => "Boolean",
        }
    }
}

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
        get_desc(Card::Return),
        get_desc(Card::ScalarNil),
        get_desc(Card::CreateTable),
        get_desc(Card::Abort),
        get_desc(Card::Len),
        get_desc(Card::SetProperty),
        get_desc(Card::GetProperty),
        get_desc(Card::ScalarInt(Default::default())),
        get_desc(Card::ScalarFloat(Default::default())),
        get_desc(Card::StringLiteral(Default::default())),
        get_desc(Card::IfTrue(Default::default())),
        get_desc(Card::IfFalse(Default::default())),
        get_desc(Card::IfElse {
            then: Default::default(),
            r#else: Default::default(),
        }),
        get_desc(Card::Jump(Default::default())),
        get_desc(Card::SetGlobalVar(Default::default())),
        get_desc(Card::SetVar(Default::default())),
        get_desc(Card::ReadVar(Default::default())),
        get_desc(Card::Repeat(Default::default())),
        get_desc(Card::While(Default::default())),
        get_desc(Card::ForEach {
            variable: Default::default(),
            lane: Default::default(),
        }),
    ]
}

#[inline(always)]
fn get_desc(node: Card) -> SubProgram<'static> {
    match node {
        Card::CallNative(_) => unreachable!(),
        Card::GetProperty => subprogram_description!(
            "GetProperty",
            "Gets a named field in the given table. Returns `nil` if the field does not exist",
            SubProgramType::Object,
            [PropertyName::Object.to_str(), PropertyName::Variable.to_str()],
            [PropertyName::Value.to_str()],
            []
        ),
        Card::SetProperty => subprogram_description!(
            "SetProperty",
            r#"Sets a named field in the given table to the input value
Order of parameters: Table, Property-Key, Value"#,
            SubProgramType::Object,
            [PropertyName::Object.to_str(), PropertyName::Variable.to_str(), PropertyName::Value.to_str()],
            [],
            []
        ),
        Card::CreateTable => subprogram_description!(
            "CreateTable",
            "Initializes a new data table",
            SubProgramType::Object,
            [],
            [],
            []
        ),
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
            [PropertyName::Value.to_str()],
            [PropertyName::Boolean.to_str()],
            []
        ),
        Card::And => subprogram_description!(
            "And",
            "Logical And",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Boolean.to_str()],
            []
        ),
        Card::Or => subprogram_description!(
            "Or",
            "Logical inclusive Or",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Boolean.to_str()],
            []
        ),
        Card::Xor => subprogram_description!(
            "Xor",
            "Logical exclusive Or",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Boolean.to_str()],
            []
        ),
        Card::Add => subprogram_description!(
            "Add",
            "Add two scalars",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::Sub => subprogram_description!(
            "Sub",
            "Subtract two scalars",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::Mul => subprogram_description!(
            "Mul",
            "Multiply two scalars",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::Div => subprogram_description!(
            "Div",
            "Divide two scalars",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::CopyLast => subprogram_description!(
            "CopyLast",
            "Duplicate the last item on the stack",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str()],
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            []
        ),
        Card::Less => subprogram_description!(
            "Less",
            "Return 1 if the first input is less than the second, 0 otherwise",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::LessOrEq => subprogram_description!(
            "LessOrEq",
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::Equals => subprogram_description!(
            "Equals",
            "Return 1 if the inputs are equal, 0 otherwise",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::ScalarNil => subprogram_description!(
            "ScalarNil",
            "Push a `Nil` value onto the stack",
            SubProgramType::Instruction,
            [],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::NotEquals => subprogram_description!(
            "NotEquals",
            "Return 0 if the inputs are equal, 1 otherwise",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str(), PropertyName::Number.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),

        Card::Pop => subprogram_description!(
            "Pop",
            "Pops the top elements on the stack and discards it",
            SubProgramType::Instruction,
            [PropertyName::Number.to_str()],
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
            [PropertyName::Integer.to_str()]
        ),

        Card::ScalarInt(_) => subprogram_description!(
            "ScalarInt",
            "Make an integer",
            SubProgramType::Instruction,
            [],
            [PropertyName::Number.to_str()],
            [PropertyName::Integer.to_str()]
        ),

        Card::ScalarFloat(_) => subprogram_description!(
            "ScalarFloat",
            "Make a real number",
            SubProgramType::Instruction,
            [],
            [PropertyName::Number.to_str()],
            [PropertyName::Float.to_str()]
        ),

        Card::StringLiteral(_) => subprogram_description!(
            "StringLiteral",
            "Make a text",
            SubProgramType::Instruction,
            [],
            [PropertyName::Number.to_str()],
            [PropertyName::Text.to_str()]
        ),

        Card::IfTrue(_) => subprogram_description!(
            "IfTrue",
            "Jump to the input lane if the last value is true else do nothing.",
            SubProgramType::Branch,
            [PropertyName::Number.to_str()],
            [],
            [PropertyName::Text.to_str()]
        ),

        Card::IfFalse(_) => subprogram_description!(
            "IfFalse",
            "Jump to the input lane if the last value is false else do nothing.",
            SubProgramType::Branch,
            [PropertyName::Number.to_str()],
            [],
            [PropertyName::Text.to_str()]
        ),

        Card::IfElse { .. } => subprogram_description!(
            "IfElse",
            "Jump to the input lane if the last value is true else jump to the second input lane.",
            SubProgramType::Branch,
            [PropertyName::Number.to_str()],
            [],
            [PropertyName::Text.to_str(), PropertyName::Text.to_str()]
        ),

        Card::Jump(_) => subprogram_description!(
            "Jump",
            "Jump to the input lane.",
            SubProgramType::Branch,
            [],
            [],
            [PropertyName::Text.to_str()]
        ),

        Card::SetGlobalVar(_) => subprogram_description!(
            "SetGlobalVar",
            "Sets the value of a global variable",
            SubProgramType::Instruction,
            [PropertyName::Object.to_str()],
            [],
            [PropertyName::Variable.to_str()]
        ),
        Card::SetVar(_) => subprogram_description!(
            "SetLocalVar",
            "Sets the value of a local variable. Local variables are only usable in the Lane they were created in.",
            SubProgramType::Instruction,
            [PropertyName::Object.to_str()],
            [],
            [PropertyName::Variable.to_str()]
        ),

        Card::ReadVar(_) => subprogram_description!(
            "ReadVar",
            "Read the value of a variable. If the variable does not exist yet it will attempt to read a global variable with the same name",
            SubProgramType::Instruction,
            [],
            [PropertyName::Object.to_str()],
            [PropertyName::Variable.to_str()]
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
            [PropertyName::Number.to_str()],
            [],
            [PropertyName::Text.to_str()]
        ),

        Card::While(_) => subprogram_description!(
            "While",
            "Repeat a lane until the lane's last value is 0",
            SubProgramType::Branch,
            [],
            [],
            [PropertyName::Text.to_str()]
        ),

        Card::ForEach {..} => subprogram_description!(
            "ForEach",
            "Repeat a lane for each key in the given table, passing the key to the lane as an argument",
            SubProgramType::Branch,
            [],
            [],
            [PropertyName::Text.to_str(), PropertyName::Text.to_str()]
        ),

        Card::Len => subprogram_description!(
            "Len",
            "Pushes the length of a table (number of keys) onto the stack",
            SubProgramType::Branch,
            [PropertyName::Object.to_str()],
            [PropertyName::Number.to_str()],
            []
        ),
    }
}
