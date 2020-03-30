//! Compiles Graphs with vertices of `AstNode` into _caol-lang_ bytecode.
//! Programs must start with a `Start` instruction.
//!
mod astnode;
mod compilation_error;
#[cfg(test)]
mod tests;
use crate::{
    traits::ByteEncodeProperties, CompiledProgram, InputString, Instruction, Label, Labels,
    INPUT_STR_LEN,
};
pub use astnode::*;
pub use compilation_error::*;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Nodes = BTreeMap<NodeId, AstNode>;

impl ByteEncodeProperties for InputString {
    const BYTELEN: usize = INPUT_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Vec<u8> {
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.chars().map(|c| c as u8));
        rr
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let len = i32::decode(bytes)?;
        let mut res = Self::new();
        for byte in bytes
            .iter()
            .skip(i32::BYTELEN)
            .take(len as usize)
            .map(|c| *c as char)
        {
            res.push(byte);
        }
        Some(res)
    }
}

/// Single compilation_unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompilationUnit {
    pub nodes: Nodes,
}

pub struct Compiler {
    compilation_unit: CompilationUnit,
    program: CompiledProgram,
}

pub fn compile(compilation_unit: CompilationUnit) -> Result<CompiledProgram, CompilationError> {
    debug!("compilation start");
    if compilation_unit.nodes.is_empty() {
        return Err(CompilationError::EmptyProgram);
    }
    let mut compiler = Compiler {
        compilation_unit,
        program: CompiledProgram::default(),
    };
    let start = compiler
        .compilation_unit
        .nodes
        .iter()
        .find(|(_, v)| match v.node.instruction() {
            Instruction::Start => true,
            _ => false,
        })
        .ok_or_else(|| CompilationError::NoStart)?;

    let mut nodes = compiler
        .compilation_unit
        .nodes
        .iter()
        .map(|(k, _)| *k)
        .collect::<HashSet<_>>();
    let mut todo = VecDeque::<i32>::with_capacity(compiler.compilation_unit.nodes.len());
    todo.push_back(*start.0);
    let mut seen = HashSet::with_capacity(compiler.compilation_unit.nodes.len());

    loop {
        while !todo.is_empty() {
            let current = todo.pop_front().unwrap();
            debug!("procesing node {:?}", current);
            nodes.remove(&current);
            seen.insert(current);
            process_node(current, &compiler.compilation_unit, &mut compiler.program)?;
            match compiler.compilation_unit.nodes[&current].child.as_ref() {
                None => compiler.program.bytecode.push(Instruction::Exit as u8),
                Some(node) => {
                    if !seen.contains(node) {
                        todo.push_front(*node);
                    } else {
                        debug!(
                            "child node of node {:?} already visited: {:?}",
                            current, node
                        );
                        compiler.program.bytecode.push(Instruction::Jump as u8);
                        compiler.program.bytecode.append(&mut node.encode());
                    }
                }
            }
        }
        match nodes.iter().next() {
            Some(node) => todo.push_back(*node),
            None => break,
        }
    }

    check_post_invariants(&compiler)?;
    debug!("compilation end");
    Ok(compiler.program)
}

fn check_post_invariants(compiler: &Compiler) -> Result<(), CompilationError> {
    debug!("checking invariants post compile");
    for (nodeid, node) in compiler.compilation_unit.nodes.iter() {
        match node.node {
            InstructionNode::Jump(ref jump) | InstructionNode::JumpIfTrue(ref jump) => {
                check_jump_post_conditions(*nodeid, jump, &compiler.program.labels)?;
            }
            _ => {}
        }
    }
    debug!("checking invariants post compile done");
    Ok(())
}

fn check_jump_post_conditions(
    nodeid: NodeId,
    jump: &JumpNode,
    labels: &Labels,
) -> Result<(), CompilationError> {
    if jump.nodeid == nodeid {
        return Err(CompilationError::InvalidJump {
            src: nodeid,
            dst: nodeid,
            msg: Some(format!(
                "Node {} is trying to jump to its own position. This is not allowed!",
                nodeid
            )),
        });
    }
    if !labels.contains_key(&jump.nodeid) {
        return Err(CompilationError::InvalidJump {
            src: nodeid,
            dst: jump.nodeid,
            msg: Some(format!(
                "Node {} is trying to jump to Non existing Node {}!",
                nodeid, jump.nodeid
            )),
        });
    }

    Ok(())
}

fn push_node(nodeid: NodeId, compilation_unit: &CompilationUnit, program: &mut CompiledProgram) {
    if let Some(node) = &compilation_unit.nodes.get(&nodeid) {
        program.bytecode.push(node.node.instruction() as u8);
    }
}

fn process_node(
    nodeid: NodeId,
    compilation_unit: &CompilationUnit,
    program: &mut CompiledProgram,
) -> Result<(), CompilationError> {
    use InstructionNode::*;

    let node = compilation_unit
        .nodes
        .get(&nodeid)
        .ok_or_else(|| CompilationError::MissingNode(nodeid))?
        .clone();

    let fromlabel =
        u32::try_from(program.bytecode.len()).expect("bytecode length to fit into 32 bits");
    program
        .labels
        .insert(nodeid, Label::new(fromlabel, fromlabel));

    let instruction = node.node;

    match instruction {
        Pop | Equals | Less | LessOrEq | NotEquals | Exit | Start | Pass | CopyLast | Add | Sub
        | Mul | Div => {
            push_node(nodeid, compilation_unit, program);
        }
        ReadVar(variable) | SetVar(variable) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut variable.name.encode());
        }
        JumpIfTrue(j) | Jump(j) => {
            let label = j.nodeid;
            if label == nodeid {
                return Err(CompilationError::InvalidJump {
                    src: nodeid,
                    dst: nodeid,
                    msg: Some(format!(
                        "Node {:?} is trying to Jump to its own location which is not supported",
                        nodeid
                    )),
                });
            }
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut label.encode());
        }
        StringLiteral(c) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut c.value.encode());
        }
        Call(c) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut c.function.encode());
        }
        ScalarArray(n) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut n.value.encode());
        }
        ScalarLabel(s) | ScalarInt(s) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut s.value.encode());
        }
        ScalarFloat(s) => {
            push_node(nodeid, compilation_unit, program);
            program.bytecode.append(&mut s.value.encode());
        }
    }
    Ok(())
}
