//! Compiles Graphs with vertices of `AstNode` into _caol-lang_ bytecode.
//! Programs must start with a `Start` instruction.
//!
mod astnode;
mod compilation_error;
pub mod description;

#[cfg(test)]
mod tests;

use crate::{
    traits::{ByteEncodeProperties, StringDecodeError},
    CompiledProgram, InputString, Instruction, Label, Labels, INPUT_STR_LEN_IN_BYTES,
};
pub use astnode::*;
pub use compilation_error::*;
use serde::{Deserialize, Serialize};
use slog::debug;
use slog::{o, Drain, Logger};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Nodes = HashMap<NodeId, AstNode>;

impl ByteEncodeProperties for InputString {
    const BYTELEN: usize = INPUT_STR_LEN_IN_BYTES;
    type DecodeError = StringDecodeError;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Vec<u8> {
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.as_bytes());
        rr
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        let len = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        const BYTELEN: usize = i32::BYTELEN;
        if bytes.len() < BYTELEN + len {
            return Err(StringDecodeError::LengthError((BYTELEN + len) as i32));
        }
        let res = std::str::from_utf8(&bytes[BYTELEN..BYTELEN + len as usize])
            .map_err(|e| StringDecodeError::Utf8DecodeError(e))?;
        Self::from(res).map_err(|_| StringDecodeError::CapacityError(Self::BYTELEN))
    }
}

/// Single compilation_unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompilationUnit {
    pub nodes: Nodes,
    pub sub_programs: Option<HashMap<String, SubProgram>>,
}

impl CompilationUnit {
    pub fn with_node(mut self, id: i32, node: AstNode) -> Self {
        self.nodes.insert(id, node);
        self
    }
}

/// Subprograms are groups of nodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubProgram {
    pub start: NodeId,
}

pub struct Compiler {
    logger: Logger,
    compilation_unit: CompilationUnit,
    program: CompiledProgram,
}

pub fn compile(
    logger: impl Into<Option<Logger>>,
    compilation_unit: CompilationUnit,
) -> Result<CompiledProgram, CompilationError> {
    let logger = logger
        .into()
        .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));

    debug!(logger, "compilation start");
    if compilation_unit.nodes.is_empty() {
        return Err(CompilationError::EmptyProgram);
    }
    let mut compiler = Compiler {
        logger,
        compilation_unit,
        program: CompiledProgram::default(),
    };
    let start = compiler
        .compilation_unit
        .nodes
        .iter()
        .find(|(_, v)| match v.node {
            InstructionNode::Start => true,
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
            debug!(compiler.logger, "procesing node {:?}", current);
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
                            compiler.logger,
                            "child node of node {:?} already visited: {:?}", current, node
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
    debug!(compiler.logger, "compilation end");
    Ok(compiler.program)
}

fn check_post_invariants(compiler: &Compiler) -> Result<(), CompilationError> {
    debug!(compiler.logger, "checking invariants post compile");
    for (nodeid, node) in compiler.compilation_unit.nodes.iter() {
        match node.node {
            InstructionNode::Jump(ref jump) | InstructionNode::JumpIfTrue(ref jump) => {
                check_jump_post_conditions(*nodeid, jump, &compiler.program.labels)?;
            }
            _ => {}
        }
    }
    debug!(compiler.logger, "checking invariants post compile done");
    Ok(())
}

fn check_jump_post_conditions(
    nodeid: NodeId,
    jump: &JumpNode,
    labels: &Labels,
) -> Result<(), CompilationError> {
    if jump.0 == nodeid {
        return Err(CompilationError::InvalidJump {
            src: nodeid,
            dst: nodeid,
            msg: Some(format!(
                "Node {} is trying to jump to its own position. This is not allowed!",
                nodeid
            )),
        });
    }
    if !labels.contains_key(&jump.0) {
        return Err(CompilationError::InvalidJump {
            src: nodeid,
            dst: jump.0,
            msg: Some(format!(
                "Node {} is trying to jump to Non existing Node {}!",
                nodeid, jump.0
            )),
        });
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum PushError {
    NoInstruction,
    NodeNotFound,
}

fn push_node(
    nodeid: NodeId,
    compilation_unit: &CompilationUnit,
    program: &mut CompiledProgram,
) -> Result<(), PushError> {
    compilation_unit
        .nodes
        .get(&nodeid)
        .ok_or(PushError::NodeNotFound)
        .and_then(|node| {
            program
                .bytecode
                .push(node.node.instruction().ok_or(PushError::NoInstruction)? as u8);
            Ok(())
        })
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
        | Mul | Div | ClearStack => {
            push_node(nodeid, compilation_unit, program).unwrap();
        }
        ReadVar(variable) | SetVar(variable) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut variable.0.encode());
        }
        JumpIfTrue(j) | Jump(j) => {
            let label = j.0;
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
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut label.encode());
        }
        StringLiteral(c) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut c.0.encode());
        }
        Call(c) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut c.0.encode());
        }
        ScalarArray(n) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut n.0.encode());
        }
        ScalarLabel(s) | ScalarInt(s) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut s.0.encode());
        }
        ScalarFloat(s) => {
            push_node(nodeid, compilation_unit, program).unwrap();
            program.bytecode.append(&mut s.0.encode());
        }
        SubProgram(b) => {
            let name = b.0;
            let sub_program = compilation_unit
                .sub_programs
                .as_ref()
                .ok_or_else(|| CompilationError::MissingSubProgram(name))?
                .get(name.as_str())
                .ok_or_else(|| CompilationError::MissingSubProgram(name))?;
            let nodeid = sub_program.start;
            compilation_unit
                .nodes
                .get(&nodeid)
                .ok_or(CompilationError::MissingNode(nodeid))
                .and_then(|_| {
                    program.bytecode.push(Instruction::Jump as u8);
                    program.bytecode.extend_from_slice(&nodeid.encode());
                    Ok(())
                })?;
        }
    }
    Ok(())
}
