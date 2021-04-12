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
    /// Push a `null` value onto the stack
    ScalarNull,
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
    /// Set the value in position given by the instruction to the value on top of the stack
    SetLocalVar,
    /// Read the value in position given by the instruction
    ReadLocalVar,
    /// Clears the last callframe's stack
    ClearStack,
    /// Returns to right-after-the-last-call-instruction
    /// Also clears the stack until the last call frame
    ///
    /// Pops the stack and pushes the value back after clearing.
    Return,
    /// Swaps the last two values on the stack
    SwapLast,
    And,
    Or,
    Xor,
    Not,
    /// Read bytecode position and move there
    Goto,
    /// Pop a scalar from the stack and `goto` there if the value was
    /// truthy
    GotoIfTrue,
    /// Pop a scalar from the stack and `goto` there if the value was
    /// falsy
    GotoIfFalse,
}
