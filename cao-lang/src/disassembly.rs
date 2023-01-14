use num_enum::TryFromPrimitive;
use std::fmt::Write;

use crate::{instruction::Instruction, prelude::CaoCompiledProgram};

pub fn disassemble(program: &CaoCompiledProgram) -> String {
    let mut result = String::with_capacity(program.bytecode.len() * 20);
    let mut i = 0;
    while i < program.bytecode.len() {
        let instr = Instruction::try_from_primitive(program.bytecode[i]);
        match instr {
            Ok(instr) => {
                writeln!(&mut result, "{:?}", instr).unwrap();
                i += instr.span();
            }
            Err(err) => {
                writeln!(&mut result, "<Invalid instruction: {err:?}>").unwrap();
                i += 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::compiler::{compile, Function, Module};

    use super::*;

    #[test]
    fn basic_disassembly_test() {
        let program = Module {
            functions: vec![("main".to_string(), Function::default())],
            ..Default::default()
        };

        let prog = compile(program, None).expect("compile");
        let dis = disassemble(&prog);

        panic!("\n{dis}");
    }
}
