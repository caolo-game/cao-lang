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
    /// Push a `nil` value onto the stack
    ScalarNil,
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
    /// Read bytecode position and Lane arity from the program and perform a jump there.
    CallLane,
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
    /// Creates a new Cao-Lang Table and pushes it onto the stack
    InitTable,
    /// Pops an Object instance from the stack, get's its value at the encoded key and pushes it's value to the stack
    GetProperty,
    /// Pops an Object instance and a Value from the stack then sets the table value at the encoded
    /// key
    SetProperty,
}
