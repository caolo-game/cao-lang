/// Single instruction of the interpreter
#[derive(Debug, Clone, Copy, Eq, PartialEq, num_enum::TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Instruction {
    /// Add two numbers
    Add,
    /// Subtract two numbers
    Sub,
    /// Multiply two numbers
    Mul,
    /// Divide the first number by the second
    Div,
    /// Call a function provided by the runtime
    /// Requires function name as a string as input
    Call,
    /// Push an int onto the stack
    ScalarInt,
    /// Push a float onto the stack
    ScalarFloat,
    /// Push a label onto the stack
    ScalarLabel,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array onto the stack
    ScalarArray,
    /// Writes the strings followed by the instruction to memory and pushes the pointer pointing to
    /// it onto the stack
    StringLiteral,
    /// Empty instruction that has no effects
    Pass,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast,
    /// If the value at the top of the stack is truthy jumps to the input node
    /// Else does nothing
    /// Quit the program returning the last value on the stack
    Exit,
    // TODO: replace jump instructions with an instruction to read label as bytecode position and
    // use GOTO instead of jumps...
    /// Jump to the label on top of the stack
    Jump,
    /// Compares two scalars
    Equals,
    /// Compares two scalars
    NotEquals,
    /// Is the first param less than the second?
    Less,
    /// Is the first param less than or equal to the second?
    LessOrEq,
    /// Pops the top of the stack and discards it
    Pop,
    /// Sets the variable at the top of the stack to the value of the second item on the stack
    SetGlobalVar,
    /// Reads the variable and pushes its value onto the stack
    ReadGlobalVar,
    /// Clears until the last sentinel
    ClearStack,
    /// If the value at the top of the stack is falsy jumps to the input node
    /// Else does nothing
    /// Insert a history entry
    Breadcrumb,
    /// Push a `null` value onto the stack
    ScalarNull,
    /// Returns to right-after-the-last-call-instruction
    /// Also clears the stack until the last sentinel
    Return,
    /// Pop an offset from the stack and remember the location to that offset from the current
    /// position
    Remember,
    /// Read bytecode position and move there
    Goto,
    /// Swaps the last two values on the stack
    SwapLast,
    /// Pop a scalar from the stack and `goto` there if the value was
    /// truthy
    GotoIfTrue,
    And,
    Or,
    Xor,
    Not,
    GotoIfFalse,
}
