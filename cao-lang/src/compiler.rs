mod card;
mod compilation_error;
mod compile_options;

pub mod description;

#[cfg(test)]
mod tests;

use crate::{
    collections::pre_hash_map::Key,
    program::{CompiledProgram, Label},
    traits::{ByteDecodeProperties, ByteEncodeProperties, ByteEncodeble, StringDecodeError},
    InputString, Instruction,
};
use crate::{NodeId, VariableId};
pub use card::*;
pub use compilation_error::*;
pub use compile_options::*;
use serde::{Deserialize, Serialize};
use slog::{debug, info};
use slog::{o, Drain, Logger};
use std::fmt::Debug;
use std::{cell::RefCell, convert::TryFrom};
use std::{collections::HashMap, mem};
use std::{
    convert::{Infallible, TryInto},
    str::FromStr,
};

impl ByteEncodeble for InputString {
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

    fn decode(bytes: &[u8]) -> Result<(usize, Self), Self::DecodeError> {
        let (ll, len) = i32::decode(bytes).map_err(|_| StringDecodeError::LengthDecodeError)?;
        let len = usize::try_from(len).map_err(|_| StringDecodeError::LengthError(len))?;
        if bytes.len() < ll + len {
            return Err(StringDecodeError::LengthError((ll + len) as i32));
        }
        let res = std::str::from_utf8(&bytes[ll..ll + len])
            .map_err(StringDecodeError::Utf8DecodeError)?;
        Self::from(res)
            .map_err(|_| StringDecodeError::CapacityError(ll + len))
            .map(|res| (ll + len, res))
    }

    unsafe fn decode_unsafe(bytes: &[u8]) -> (usize, Self) {
        let (ll, len) = i32::decode_unsafe(bytes);
        let len = len as usize;
        let res = std::str::from_utf8_unchecked(&bytes[ll..ll + len]);
        Self::from(res).map(|res| (ll + len, res)).unwrap()
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

    /// maps lane names to their indices
    pub jump_table: HashMap<String, Key>,

    pub options: CompileOptions,
    pub program: CompiledProgram,
    pub next_var: RefCell<VariableId>,
    _m: std::marker::PhantomData<&'a ()>,
}

pub fn compile(
    logger: impl Into<Option<Logger>>,
    compilation_unit: CompilationUnit,
    compile_options: impl Into<Option<CompileOptions>>,
) -> Result<CompiledProgram, CompilationError> {
    let mut compiler = Compiler::new(logger);
    compiler.compile(compilation_unit, compile_options)
}

impl<'a> Compiler<'a> {
    /// If no `logger` is provided, falls back to the 'standard' log crate.
    pub fn new<L: Into<Option<Logger>>>(logger: L) -> Self {
        fn _new<'a>(logger: Logger) -> Compiler<'a> {
            Compiler {
                logger,
                program: CompiledProgram::default(),
                jump_table: Default::default(),
                options: Default::default(),
                next_var: RefCell::new(VariableId(0)),
                _m: Default::default(),
            }
        }

        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));

        _new(logger)
    }

    pub fn compile(
        &mut self,
        compilation_unit: CompilationUnit,
        compile_options: impl Into<Option<CompileOptions>>,
    ) -> Result<CompiledProgram, CompilationError> {
        self.options = compile_options.into().unwrap_or_default();
        self._compile(compilation_unit)
    }

    fn _compile(
        &mut self,
        compilation_unit: CompilationUnit,
    ) -> Result<CompiledProgram, CompilationError> {
        info!(self.logger, "compilation start");
        if compilation_unit.lanes.is_empty() {
            return Err(CompilationError::EmptyProgram);
        }
        // check if len fits in 16 bits
        let _: u16 = match compilation_unit.lanes.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(CompilationError::TooManyLanes),
        };

        self.program = CompiledProgram::default();

        let mut lanes = Vec::with_capacity(compilation_unit.lanes.len());
        for (i, n) in compilation_unit.lanes.into_iter().enumerate() {
            if self.jump_table.contains_key(n.name.as_str()) {
                return Err(CompilationError::DuplicateName(n.name));
            }
            lanes.push((i, n.cards));
            self.jump_table.insert(
                n.name,
                Key::from_un32(
                    NodeId {
                        // we know that i fits in 16 bits from the check above
                        lane: i as u16,
                        pos: 0,
                    }
                    .into(),
                ),
            );
        }

        for (il, lane) in lanes {
            info!(self.logger, "procesing lane #{}", il);
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
                debug!(self.logger, "procesing card {:?}", nodeid);
                self.process_node(nodeid, card)?;
            }
            // insert exit node, so execution stops even if the bytecode contains
            // additional cards after this lane...
            // also, this is why empty lanes are valid
            self.process_node(
                NodeId {
                    lane: il as u16,
                    pos: len,
                },
                Card::ExitWithCode(card::IntegerNode(0)),
            )?;
        }

        info!(self.logger, "compilation end");
        Ok(mem::take(&mut self.program))
    }

    pub fn process_node(&mut self, nodeid: NodeId, card: Card) -> Result<(), CompilationError> {
        use Card::*;

        let ptr = u32::try_from(self.program.bytecode.len())
            .expect("bytecode length to fit into 32 bits");
        let nodeid_hash = Key::from_un32(nodeid.into());
        self.program.labels.0.insert(nodeid_hash, Label::new(ptr));

        if let Some(instr) = card.instruction() {
            if self.options.breadcrumbs {
                self.program.bytecode.push(Instruction::Breadcrumb as u8);
                nodeid.encode(&mut self.program.bytecode).unwrap();
                // instr for the breadcrumb
                self.program.bytecode.push(instr as u8);
            }
            // instruction itself
            self.program.bytecode.push(instr as u8);
        }
        match card {
            Pop | Equals | Less | LessOrEq | NotEquals | Exit | Pass | CopyLast | Add | Sub
            | Mul | Div | ClearStack => {}
            ReadVar(variable) | SetVar(variable) => {
                let mut next_var = self.next_var.borrow_mut();
                let varhash = Key::from_bytes(variable.0.as_bytes());

                let id = self
                    .program
                    .variables
                    .0
                    .entry(varhash)
                    .or_insert_with(move || {
                        let id = *next_var;
                        *next_var = VariableId(id.0 + 1);
                        id
                    });
                id.encode(&mut self.program.bytecode).unwrap();
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
                to.encode(&mut self.program.bytecode).unwrap();
            }
            StringLiteral(c) => {
                c.0.encode(&mut self.program.bytecode).unwrap();
            }
            Call(c) => {
                let name = &c.0;
                let key = Key::from_str(name.as_str()).unwrap();
                key.encode(&mut self.program.bytecode).unwrap();
            }
            ScalarArray(n) => {
                n.0.encode(&mut self.program.bytecode).unwrap();
            }
            ExitWithCode(s) => {
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                s.0.encode(&mut self.program.bytecode).unwrap();
                self.program.bytecode.push(Instruction::Exit as u8);
            }
            ScalarLabel(s) | ScalarInt(s) => {
                s.0.encode(&mut self.program.bytecode).unwrap();
            }
            ScalarFloat(s) => {
                s.0.encode(&mut self.program.bytecode).unwrap();
            }
        }
        Ok(())
    }
}