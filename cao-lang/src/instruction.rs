use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, mem::transmute};

/// Single instruction of the interpreter
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
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
    /// Quit the program
    /// Implicitly inserted by the compiler after every leaf node
    Exit = 13,
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
    SetVar = 20,
    /// Reads the variable and pushes its value onto the stack
    ReadVar = 21,

    ClearStack = 22,
    /// If the value at the top of the stack is falsy jumps to the input node
    /// Else does nothing
    JumpIfFalse = 23,
    /// Insert a history entry
    Breadcrumb = 24,
}

impl Instruction {
    pub fn is_valid_instr(n: u8) -> bool {
        n <= 24
    }
}

impl TryFrom<u8> for Instruction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if Self::is_valid_instr(value) {
            unsafe { Ok(transmute(value)) }
        } else {
            Err(value)
        }
    }
}
