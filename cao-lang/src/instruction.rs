use std::{convert::TryFrom, mem::transmute};

/// Single instruction of the interpreter
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Instruction {
    /// Add two numbers
    Add = 0,
    /// Subtract two numbers
    Sub = 1,
    /// Multiply two numbers
    Mul = 2,
    /// Divide the first number by the second
    Div = 3,
    /// Call a function provided by the runtime
    /// Requires function name as a string as input
    Call = 4,
    /// Push an int onto the stack
    ScalarInt = 5,
    /// Push a float onto the stack
    ScalarFloat = 6,
    /// Push a label onto the stack
    ScalarLabel = 7,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array onto the stack
    ScalarArray = 8,
    /// Writes the strings followed by the instruction to memory and pushes the pointer pointing to
    /// it onto the stack
    StringLiteral = 9,
    /// Empty instruction that has no effects
    Pass = 10,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast = 11,
    /// If the value at the top of the stack is truthy jumps to the input node
    /// Else does nothing
    JumpIfTrue = 12,
    /// Quit the program returning the last value on the stack
    Exit = 13,
    // TODO: replace jump instructions with an instruction to read label as bytecode position and
    // use GOTO instead of jumps...
    /// Jump to the label on top of the stack
    Jump = 14,
    /// Compares two scalars
    Equals = 15,
    /// Compares two scalars
    NotEquals = 16,
    /// Is the first param less than the second?
    Less = 17,
    /// Is the first param less than or equal to the second?
    LessOrEq = 18,
    /// Pops the top of the stack and discards it
    Pop = 19,
    /// Sets the variable at the top of the stack to the value of the second item on the stack
    SetGlobalVar = 20,
    /// Reads the variable and pushes its value onto the stack
    ReadGlobalVar = 21,
    /// Clears until the last sentinel
    ClearStack = 22,
    /// If the value at the top of the stack is falsy jumps to the input node
    /// Else does nothing
    JumpIfFalse = 23,
    /// Insert a history entry
    Breadcrumb = 24,
    /// Push a `null` value onto the stack
    ScalarNull = 25,
    /// Returns to right-after-the-last-call-instruction
    /// Also clears the stack until the last sentinel
    Return = 26,
    /// Pop an offset from the stack and remember the location to that offset from the current
    /// position
    Remember = 27,
    /// Starts a new scope
    ScopeStart = 28,
    /// Starts a new scope
    ScopeEnd = 29,
    /// Pop a bytecode position from the stack and `goto` there
    Goto = 30,
    /// Swaps the last two values on the stack
    SwapLast = 31,
    /// Pop a bytecode position and a scalar from the stack and `goto` there if the value was
    /// truthy
    GotoIfTrue = 32,
    And = 33,
    Or = 34,
    Xor = 35,
}

impl Instruction {
    pub fn is_valid_instr(n: u8) -> bool {
        n <= 35
    }
}

impl TryFrom<u8> for Instruction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if Self::is_valid_instr(value) {
            // # SAFETY
            // as long as the values are continous and fit in 8 bits this is fine (tm)
            unsafe { Ok(transmute(value)) }
        } else {
            Err(value)
        }
    }
}
