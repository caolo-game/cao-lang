mod card;
mod compilation_error;
pub mod description;

#[cfg(test)]
mod tests;

use crate::NodeId;
use crate::{
    program::{CompiledProgram, Label},
    traits::{ByteDecodeProperties, ByteEncodeProperties, ByteEncodeble, StringDecodeError},
    InputString, Instruction, INPUT_STR_LEN_IN_BYTES,
};
pub use card::*;
pub use compilation_error::*;
use serde::{Deserialize, Serialize};
use slog::{debug, info};
use slog::{o, Drain, Logger};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::{Infallible, TryInto};
use std::fmt::Debug;

impl ByteEncodeble for InputString {
    const BYTELEN: usize = INPUT_STR_LEN_IN_BYTES;
    fn displayname() -> &'static str {
        "Text"
    }
}

impl ByteEncodeProperties for InputString {
    type EncodeError = Infallible;
    fn encode(self, rr: &mut Vec<u8>) -> Result<(), Self::EncodeError> {
        (self.len() as i32).encode(rr)?;
        rr.extend(self.as_bytes());
        Ok(())
    }
}

impl ByteDecodeProperties for InputString {
    type DecodeError = StringDecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> {
        let len = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        const BYTELEN: usize = <i32 as ByteEncodeble>::BYTELEN;
        if bytes.len() < BYTELEN + len {
            return Err(StringDecodeError::LengthError((BYTELEN + len) as i32));
        }
        let res = std::str::from_utf8(&bytes[BYTELEN..BYTELEN + len])
            .map_err(StringDecodeError::Utf8DecodeError)?;
        Self::from(res).map_err(|_| StringDecodeError::CapacityError(Self::BYTELEN))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub name: String,
    pub cards: Vec<Card>,
}

/// Single compilation unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompilationUnit {
    pub lanes: Vec<Lane>,
}

pub struct Compiler<'a> {
    pub logger: Logger,
    pub program: CompiledProgram,

    /// maps lane names to their indices
    pub jump_table: HashMap<String, NodeId>,

    _m: std::marker::PhantomData<&'a ()>,
}

pub fn compile(
    logger: impl Into<Option<Logger>>,
    compilation_unit: CompilationUnit,
) -> Result<CompiledProgram, CompilationError> {
    let logger = logger
        .into()
        .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));

    info!(logger, "compilation start");
    if compilation_unit.lanes.is_empty() {
        return Err(CompilationError::EmptyProgram);
    }
    // check if len fits in 16 bits
    let _: u16 = match compilation_unit.lanes.len().try_into() {
        Ok(i) => i,
        Err(_) => return Err(CompilationError::TooManyLanes),
    };
    let mut compiler = Compiler {
        logger,
        program: CompiledProgram::default(),
        jump_table: Default::default(),
        _m: Default::default(),
    };

    let mut lanes = Vec::with_capacity(compilation_unit.lanes.len());
    for (i, n) in compilation_unit.lanes.into_iter().enumerate() {
        if compiler.jump_table.contains_key(n.name.as_str()) {
            return Err(CompilationError::DuplicateName(n.name));
        }
        lanes.push((i, n.cards));
        compiler.jump_table.insert(
            n.name,
            NodeId {
                // we know that i fits in 16 bits from the check above
                lane: i as u16,
                pos: 0,
            },
        );
    }

    for (il, lane) in lanes {
        info!(compiler.logger, "procesing lane #{}", il);
        // check if len fits in 16 bits
        let len: u16 = match lane.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(CompilationError::TooManyCards(il)),
        };
        for (ic, card) in lane.into_iter().enumerate() {
            let nodeid = NodeId {
                lane: il as u16,
                pos: ic as u16,
            };
            debug!(compiler.logger, "procesing card {:?}", nodeid);
            compiler.process_node(nodeid, card)?;
        }
        // insert exit node, so execution stops even if the bytecode contains
        // additional cards after this lane...
        // also, this is why empty lanes are valid
        compiler.process_node(
            NodeId {
                lane: il as u16,
                pos: len,
            },
            Card::ExitWithCode(card::IntegerNode(0)),
        )?;
    }

    info!(compiler.logger, "compilation end");
    Ok(compiler.program)
}

impl<'a> Compiler<'a> {
    /// If no `logger` is provided, falls back to the 'standard' log crate.
    pub fn new<L: Into<Option<Logger>>>(logger: L) -> Self {
        Compiler {
            logger: logger
                .into()
                .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!())),
            program: CompiledProgram::default(),
            jump_table: Default::default(),
            _m: Default::default(),
        }
    }

    pub fn process_node(&mut self, nodeid: NodeId, card: Card) -> Result<(), CompilationError> {
        use Card::*;

        let program = &mut self.program;

        let ptr =
            u32::try_from(program.bytecode.len()).expect("bytecode length to fit into 32 bits");
        program.labels.0.insert(nodeid, Label::new(ptr));

        if let Some(instr) = card.instruction() {
            program.bytecode.push(instr as u8);
            nodeid.encode(&mut program.bytecode).unwrap();
        }
        match card {
            Pop | Equals | Less | LessOrEq | NotEquals | Exit | Pass | CopyLast | Add | Sub
            | Mul | Div | ClearStack => {}
            ReadVar(variable) | SetVar(variable) => {
                variable.0.encode(&mut program.bytecode).unwrap();
            }
            JumpIfFalse(jmp) | JumpIfTrue(jmp) | Jump(jmp) => {
                let to =
                    self.jump_table
                        .get(jmp.0.as_str())
                        .ok_or(CompilationError::InvalidJump {
                            src: nodeid,
                            dst: jmp.0,
                            msg: None,
                        })?;
                to.encode(&mut program.bytecode).unwrap();
            }
            StringLiteral(c) => {
                c.0.encode(&mut program.bytecode).unwrap();
            }
            Call(c) => {
                c.0.encode(&mut program.bytecode).unwrap();
            }
            ScalarArray(n) => {
                n.0.encode(&mut program.bytecode).unwrap();
            }
            ExitWithCode(s) => {
                program.bytecode.push(Instruction::ScalarInt as u8);
                nodeid.encode(&mut program.bytecode).unwrap();
                s.0.encode(&mut program.bytecode).unwrap();
                program.bytecode.push(Instruction::Exit as u8);
                nodeid.encode(&mut program.bytecode).unwrap();
            }
            ScalarLabel(s) | ScalarInt(s) => {
                s.0.encode(&mut program.bytecode).unwrap();
            }
            ScalarFloat(s) => {
                s.0.encode(&mut program.bytecode).unwrap();
            }
        }
        Ok(())
    }
}
