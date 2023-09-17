use std::{mem::transmute, str::FromStr};

use crate::{
    collections::{
        handle_table::{Handle, HandleTable},
        hash_map::CaoHashMap,
    },
    compiler::{CardIndex, NameSpace},
    instruction::Instruction,
    VarName,
};
use crate::{version, VariableId};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Labels(pub HandleTable<Label>);

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variables {
    pub ids: HandleTable<VariableId>,
    pub names: HandleTable<VarName>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    /// Position of this card in the bytecode of the program
    pub pos: u32,
}

impl Label {
    pub fn new(pos: u32) -> Self {
        Self { pos }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trace {
    pub namespace: NameSpace,
    pub index: CardIndex,
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ns in self.namespace.iter() {
            write!(f, "{ns}.")?;
        }
        write!(f, "{}", self.index)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CaoCompiledProgram {
    /// Instructions
    pub bytecode: Vec<u8>,
    /// Data used by instuctions with variable length inputs
    pub data: Vec<u8>,
    pub labels: Labels,
    pub variables: Variables,
    pub cao_lang_version: (u8, u8, u16),
    pub trace: CaoHashMap<u32, Trace>,
}

impl CaoCompiledProgram {
    pub fn variable_id(&self, name: &str) -> Option<VariableId> {
        self.variables
            .ids
            .get(Handle::from_str(name).unwrap())
            .copied()
    }

    pub fn print_disassembly(&self) {
        let mut pl = String::new();
        self.disassemble(&mut pl).unwrap();
        // FIXME: I'd prefer writing straight to stdout...
        println!("{pl}");
    }

    pub fn disassemble(&self, mut writer: impl std::fmt::Write) -> std::fmt::Result {
        let mut i = 0;
        while i < self.bytecode.len() {
            let instr: u8 = self.bytecode[i];
            let instr: Instruction = unsafe { transmute(instr) };
            write!(writer, "{i}\t")?;
            // TODO: also print the arguments of the instructions
            match instr {
                Instruction::Add => writeln!(writer, "Add")?,
                Instruction::Sub => writeln!(writer, "Sub")?,
                Instruction::Mul => writeln!(writer, "Mul")?,
                Instruction::Div => writeln!(writer, "Div")?,
                Instruction::CallNative => writeln!(writer, "CallNative")?,
                Instruction::ScalarInt => writeln!(writer, "ScalarInt")?,
                Instruction::ScalarFloat => writeln!(writer, "ScalarFloat")?,
                Instruction::ScalarNil => writeln!(writer, "ScalarNil")?,
                Instruction::StringLiteral => writeln!(writer, "StringLiteral")?,
                Instruction::CopyLast => writeln!(writer, "CopyLast")?,
                Instruction::Exit => writeln!(writer, "Exit")?,
                Instruction::CallFunction => writeln!(writer, "CallFunction")?,
                Instruction::Equals => writeln!(writer, "Equals")?,
                Instruction::NotEquals => writeln!(writer, "NotEquals")?,
                Instruction::Less => writeln!(writer, "Less")?,
                Instruction::LessOrEq => writeln!(writer, "LessOrEq")?,
                Instruction::Pop => writeln!(writer, "Pop")?,
                Instruction::SetGlobalVar => writeln!(writer, "SetGlobalVar")?,
                Instruction::ReadGlobalVar => writeln!(writer, "ReadGlobalVar")?,
                Instruction::SetLocalVar => writeln!(writer, "SetLocalVar")?,
                Instruction::ReadLocalVar => writeln!(writer, "ReadLocalVar")?,
                Instruction::ClearStack => writeln!(writer, "ClearStack")?,
                Instruction::Return => writeln!(writer, "Return")?,
                Instruction::SwapLast => writeln!(writer, "SwapLast")?,
                Instruction::And => writeln!(writer, "And")?,
                Instruction::Or => writeln!(writer, "Or")?,
                Instruction::Xor => writeln!(writer, "Xor")?,
                Instruction::Not => writeln!(writer, "Not")?,
                Instruction::Goto => writeln!(writer, "Goto")?,
                Instruction::GotoIfTrue => writeln!(writer, "GotoIfTrue")?,
                Instruction::GotoIfFalse => writeln!(writer, "GotoIfFalse")?,
                Instruction::InitTable => writeln!(writer, "InitTable")?,
                Instruction::GetProperty => writeln!(writer, "GetProperty")?,
                Instruction::SetProperty => writeln!(writer, "SetProperty")?,
                Instruction::Len => writeln!(writer, "Len")?,
                Instruction::BeginForEach => writeln!(writer, "BeginForEach")?,
                Instruction::ForEach => writeln!(writer, "ForEach")?,
                Instruction::FunctionPointer => writeln!(writer, "FunctionPointer")?,
                Instruction::NativeFunctionPointer => writeln!(writer, "NativeFunctionPointer")?,
                Instruction::NthRow => writeln!(writer, "NthRow")?,
                Instruction::AppendTable => writeln!(writer, "AppendTable")?,
                Instruction::PopTable => writeln!(writer, "PopTable")?,
                Instruction::Closure => writeln!(writer, "Closure")?,
                Instruction::SetUpvalue => writeln!(writer, "SetUpvalue")?,
                Instruction::ReadUpvalue => writeln!(writer, "ReadUpvalue")?,
                Instruction::RegisterUpvalue => writeln!(writer, "RegisterUpvalue")?,
                Instruction::CloseUpvalue => writeln!(writer, "CloseUpvalue")?,
            }
            i += instr.span();
        }
        Ok(())
    }
}

impl Default for CaoCompiledProgram {
    fn default() -> Self {
        Self {
            bytecode: Default::default(),
            data: Default::default(),
            labels: Default::default(),
            variables: Default::default(),
            cao_lang_version: (version::MAJOR, version::MINOR, version::PATCH),
            trace: Default::default(),
        }
    }
}
