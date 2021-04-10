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
    /// Quit the program returning the last value on the stack
    Exit = 12,
    // TODO: replace jump instructions with an instruction to read label as bytecode position and
    // use GOTO instead of jumps...
    /// Jump to the label on top of the stack
    Jump = 13,
    /// Compares two scalars
    Equals = 14,
    /// Compares two scalars
    NotEquals = 15,
    /// Is the first param less than the second?
    Less = 16,
    /// Is the first param less than or equal to the second?
    LessOrEq = 17,
    /// Pops the top of the stack and discards it
    Pop = 18,
    /// Sets the variable at the top of the stack to the value of the second item on the stack
    SetGlobalVar = 19,
    /// Reads the variable and pushes its value onto the stack
    ReadGlobalVar = 20,
    /// Clears until the last sentinel
    ClearStack = 21,
    /// If the value at the top of the stack is falsy jumps to the input node
    /// Else does nothing
    /// Insert a history entry
    Breadcrumb = 22,
    /// Push a `null` value onto the stack
    ScalarNull = 23,
    /// Returns to right-after-the-last-call-instruction
    /// Also clears the stack until the last sentinel
    Return = 24,
    /// Pop an offset from the stack and remember the location to that offset from the current
    /// position
    Remember = 25,
    /// Starts a new scope
    ScopeStart = 26,
    /// Starts a new scope
    ScopeEnd = 27,
    /// Read bytecode position and move there
    Goto = 28,
    /// Swaps the last two values on the stack
    SwapLast = 29,
    /// Pop a scalar from the stack and `goto` there if the value was
    /// truthy
    GotoIfTrue = 30,
    And = 31,
    Or = 32,
    Xor = 33,
    Not = 34,
    GotoIfFalse = 35,
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
