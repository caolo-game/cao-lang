use std::mem::size_of;

use crate::{prelude::Handle, VariableId};

/// Single instruction of the interpreter
#[derive(Debug, Clone, Copy, Eq, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum Instruction {
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
    CallNative,
    /// Push an int onto the stack
    ScalarInt,
    /// Push a float onto the stack
    ScalarFloat,
    /// Push a `nil` value onto the stack
    ScalarNil,
    /// Writes the strings followed by the instruction to memory and pushes the pointer pointing to
    /// it onto the stack
    StringLiteral,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast,
    /// Quit the program
    Exit,
    /// Read bytecode position and Function arity from the program and perform a jump there.
    CallFunction,
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
    /// Pops: `key`, `object`, `value`
    /// Sets the value of the given `key` on the `object` to `value`
    ///
    /// The reason `value` is the first to be pushed is the read/setvar shorthands
    SetProperty,
    /// Pushes the length of the topmost table to the stack
    /// Errors if the top Value is not a Table
    Len,

    BeginForEach,
    ForEach,

    FunctionPointer,
    NativeFunctionPointer,

    /// Get the given row in a Table
    NthRow,

    /// Append the given value to the Table
    AppendTable,
    /// Pop the last row from the Table
    PopTable,
    Closure,
    SetUpvalue,
    ReadUpvalue,
    RegisterUpvalue,
    CloseUpvalue,
}

impl Instruction {
    /// Returns the span of this instruction in bytecode
    #[allow(unused)]
    pub fn span(self) -> usize {
        let data_span = match self {
            Instruction::CallFunction
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::ScalarNil
            | Instruction::CopyLast
            | Instruction::Exit
            | Instruction::Equals
            | Instruction::NotEquals
            | Instruction::Less
            | Instruction::LessOrEq
            | Instruction::Pop
            | Instruction::ClearStack
            | Instruction::Return
            | Instruction::SwapLast
            | Instruction::And
            | Instruction::Or
            | Instruction::Xor
            | Instruction::Not
            | Instruction::InitTable
            | Instruction::GetProperty
            | Instruction::SetProperty
            | Instruction::Len
            | Instruction::NthRow
            | Instruction::AppendTable
            | Instruction::PopTable
            | Instruction::CloseUpvalue
            | Instruction::Add => 0,
            Instruction::CallNative => size_of::<Handle>(),
            Instruction::ScalarInt => size_of::<i64>(),
            Instruction::ScalarFloat => size_of::<f64>(),
            Instruction::StringLiteral => size_of::<u32>(),
            Instruction::NativeFunctionPointer => Instruction::StringLiteral.span(),
            Instruction::SetGlobalVar => size_of::<VariableId>(),
            Instruction::ReadGlobalVar => size_of::<VariableId>(),
            Instruction::SetLocalVar
            | Instruction::SetUpvalue
            | Instruction::ReadUpvalue
            | Instruction::ReadLocalVar => size_of::<u32>(),
            Instruction::Goto | Instruction::GotoIfTrue | Instruction::GotoIfFalse => {
                size_of::<i32>()
            }
            Instruction::BeginForEach => size_of::<u32>() * 2,
            Instruction::ForEach => size_of::<u32>() * 5,
            Instruction::FunctionPointer => size_of::<Handle>() + size_of::<u32>(),
            Instruction::Closure => size_of::<Handle>() + size_of::<u32>(),
            Instruction::RegisterUpvalue => size_of::<u8>() * 2,
        };
        1 + data_span
    }
}
