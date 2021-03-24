mod card;
mod compilation_error;
mod compile_options;

pub mod card_description;

#[cfg(test)]
mod tests;

use crate::{
    collections::pre_hash_map::{Key, PreHashMap},
    program::{CaoProgram, Label},
    traits::{ByteDecodeProperties, ByteEncodeProperties, ByteEncodeble, StringDecodeError},
    InputString, Instruction,
};
use crate::{NodeId, VariableId};
pub use card::*;
pub use compilation_error::*;
pub use compile_options::*;
use std::fmt::Debug;
use std::mem;
use std::{cell::RefCell, convert::TryFrom};
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lane {
    pub name: Option<String>,
    pub cards: Vec<Card>,
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            name: None,
            cards: Vec::new(),
        }
    }
}

impl Lane {
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_card(mut self, card: Card) -> Self {
        self.cards.push(card);
        self
    }

    /// overrides the existing cards
    pub fn with_cards<C: Into<Vec<Card>>>(mut self, cards: C) -> Self {
        self.cards = cards.into();
        self
    }
}

/// Single unit of compilation, representing a single program
///
/// Execution will begin with the first Lane
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilationUnit {
    pub lanes: Vec<Lane>,
}

pub struct Compiler<'a> {
    /// maps lane names to their NodeId keys
    pub jump_table: PreHashMap<Key>,

    pub options: CompileOptions,
    pub program: CaoProgram,
    pub next_var: RefCell<VariableId>,
    _m: std::marker::PhantomData<&'a ()>,
}

pub fn compile(
    compilation_unit: CompilationUnit,
    compile_options: impl Into<Option<CompileOptions>>,
) -> Result<CaoProgram, CompilationError> {
    let mut compiler = Compiler::new();
    compiler.compile(compilation_unit, compile_options)
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Compiler<'a> {
    /// If no `logger` is provided, falls back to the 'standard' log crate.
    pub fn new() -> Self {
        Compiler {
            program: CaoProgram::default(),
            jump_table: Default::default(),
            options: Default::default(),
            next_var: RefCell::new(VariableId(0)),
            _m: Default::default(),
        }
    }

    pub fn compile(
        &mut self,
        compilation_unit: CompilationUnit,
        compile_options: impl Into<Option<CompileOptions>>,
    ) -> Result<CaoProgram, CompilationError> {
        self.options = compile_options.into().unwrap_or_default();
        // minimize the surface of the generic function
        self._compile(compilation_unit)
    }

    fn _compile(
        &mut self,
        mut compilation_unit: CompilationUnit,
    ) -> Result<CaoProgram, CompilationError> {
        if compilation_unit.lanes.is_empty() {
            return Err(CompilationError::EmptyProgram);
        }
        // initialize
        self.program = CaoProgram::default();
        self._compile_stage_1(&mut compilation_unit)?;
        self._compile_stage_2(compilation_unit)?;

        Ok(mem::take(&mut self.program))
    }

    /// build the jump table and consume the lane names
    /// also reserve memory for the program labels
    fn _compile_stage_1(
        &mut self,
        compilation_unit: &mut CompilationUnit,
    ) -> Result<(), CompilationError> {
        // check if len fits in 16 bits
        let _: u16 = match compilation_unit.lanes.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(CompilationError::TooManyLanes),
        };

        let mut num_cards = 0usize;
        for (i, n) in compilation_unit.lanes.iter_mut().enumerate() {
            let indexkey = Key::from_u32(i as u32);
            assert!(!self.jump_table.contains(indexkey));
            num_cards += n.cards.len();

            let nodekey = Key::from_u32(
                NodeId {
                    // we know that i fits in 16 bits from the check above
                    lane: i as u16,
                    pos: 0,
                }
                .into(),
            );
            // allow referencing lanes using both name and index
            self.jump_table.insert(indexkey, nodekey);
            if let Some(ref mut name) = n.name.as_mut() {
                let namekey = Key::from_str(name.as_str()).expect("Failed to hash lane name");
                if self.jump_table.contains(namekey) {
                    return Err(CompilationError::DuplicateName(std::mem::take(name)));
                }
                self.jump_table.insert(namekey, nodekey);
            }
        }

        self.program.labels.0.reserve(num_cards);
        Ok(())
    }

    /// consume lane cards and build the bytecode
    fn _compile_stage_2(
        &mut self,
        compilation_unit: CompilationUnit,
    ) -> Result<(), CompilationError> {
        let mut lanes = compilation_unit
            .lanes
            .into_iter()
            .map(|Lane { cards, .. }| cards)
            .enumerate();

        // main lane has no enclosing scope
        if let Some((il, main_lane)) = lanes.next() {
            let len: u16 = match main_lane.len().try_into() {
                Ok(i) => i,
                Err(_) => return Err(CompilationError::TooManyCards(il)),
            };
            self._process_lane(il, main_lane, 0)?;
            let nodeid = NodeId {
                lane: il as u16,
                pos: len,
            };
            self.process_node(nodeid, Card::ExitWithCode(IntegerNode(0)))?;
        }

        for (il, lane) in lanes {
            // manually add a scope start instruction and the position information
            let nodeid = NodeId {
                lane: il as u16,
                pos: 0,
            };
            let nodeid_hash = Key::from_u32(nodeid.into());
            let handle = u32::try_from(self.program.bytecode.len())
                .expect("bytecode length to fit into 32 bits");
            self.program
                .labels
                .0
                .insert(nodeid_hash, Label::new(handle));
            self.program.bytecode.push(Instruction::ScopeStart as u8);

            // process the lane
            self._process_lane(il, lane, 1)?;

            // end the scope and return
            self.program.bytecode.push(Instruction::ScopeEnd as u8);
            self.program.bytecode.push(Instruction::Return as u8);
        }

        Ok(())
    }

    fn _process_lane(
        &mut self,
        il: usize,
        cards: Vec<Card>,
        instruction_offset: i32,
    ) -> Result<(), CompilationError> {
        // check if len fits in 16 bits
        let _len: u16 = match cards.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(CompilationError::TooManyCards(il)),
        };
        for (ic, card) in cards.into_iter().enumerate() {
            let nodeid = NodeId {
                lane: il as u16,
                pos: (ic as i32 + instruction_offset) as u16,
            };
            self.process_node(nodeid, card)?;
        }
        Ok(())
    }

    fn _encode_jump(&mut self, nodeid: NodeId, lane: &LaneNode) -> Result<(), CompilationError> {
        let to = match lane {
            LaneNode::LaneName(lane) => self
                .jump_table
                .get(Key::from_str(lane).expect("Failed to hash jump target name"))
                .ok_or(CompilationError::InvalidJump {
                    src: nodeid,
                    dst: lane.to_string(),
                    msg: None,
                })?,
            LaneNode::LaneId(id) => self.jump_table.get(Key::from_u32(*id as u32)).ok_or(
                CompilationError::InvalidJump {
                    src: nodeid,
                    dst: format!("Lane id {}", id),
                    msg: None,
                },
            )?,
        };
        to.encode(&mut self.program.bytecode).unwrap();
        Ok(())
    }

    /// push `data` into the `data section` of the program and encode a poiter to it for the current instruction
    fn _push_data<T: ByteEncodeProperties>(&mut self, data: T) -> Result<(), CompilationError> {
        let handle = self.program.data.len();
        data.encode(&mut self.program.data)
            .expect("Failed to encode data");
        let handle: u32 = handle
            .try_into()
            .expect("data handle doesn't fit into 32 bits");
        handle.encode(&mut self.program.bytecode).unwrap();
        Ok(())
    }

    pub fn process_node(&mut self, nodeid: NodeId, card: Card) -> Result<(), CompilationError> {
        let handle = u32::try_from(self.program.bytecode.len())
            .expect("bytecode length to fit into 32 bits");
        let nodeid_hash = Key::from_u32(nodeid.into());
        self.program
            .labels
            .0
            .insert(nodeid_hash, Label::new(handle));

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
            Card::While(repeat) => {
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                // 6 = sizeof scalarint + sizeof remember = right before this bit of code
                (-6i32).encode(&mut self.program.bytecode).unwrap();
                self.program.bytecode.push(Instruction::Remember as u8);
                self.program.bytecode.push(Instruction::Jump as u8);
                self._encode_jump(nodeid, &repeat)?;
                // rerun if not 0
                self.program.bytecode.push(Instruction::GotoIfTrue as u8);
            }
            Card::Repeat(repeat) => {
                // ## Semantics:
                //
                // 1) push counter + 1 on the stack
                // 2) remember this slot
                // 3) push 1
                // 4) sub
                // 5) clone last (loop counter)
                // 6) jump if true to the lane
                // 7) return to the remembered slot if true

                // 1)
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                1i32.encode(&mut self.program.bytecode).unwrap();
                self.program.bytecode.push(Instruction::Add as u8);

                let checkpoint = self.program.bytecode.len(); // 2)
                self.program.bytecode.push(Instruction::ScalarInt as u8); // 3)
                1i32.encode(&mut self.program.bytecode).unwrap();
                self.program.bytecode.push(Instruction::Sub as u8); // 4)

                // leave the loop counter on the stack after the jump and the goto
                // 5)
                self.program.bytecode.push(Instruction::CopyLast as u8);
                self.program.bytecode.push(Instruction::CopyLast as u8);
                // 2)
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                // remember the position above (+5 is the length of the remember instruction -1)
                (-((self.program.bytecode.len() + 5 - checkpoint) as i32))
                    .encode(&mut self.program.bytecode)
                    .unwrap();
                self.program.bytecode.push(Instruction::Remember as u8);
                self.program.bytecode.push(Instruction::SwapLast as u8);
                // jump to the lane if not 0
                // 6)
                self.program.bytecode.push(Instruction::JumpIfTrue as u8);
                self._encode_jump(nodeid, &repeat)?;
                // discard the lane return value
                self.program.bytecode.push(Instruction::Pop as u8);
                // rerun if not 0
                // 7)
                self.program.bytecode.push(Instruction::SwapLast as u8);
                self.program.bytecode.push(Instruction::GotoIfTrue as u8);
            }
            Card::ReadGlobalVar(variable) | Card::SetGlobalVar(variable) => {
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
            Card::JumpIfFalse(jmp) | Card::JumpIfTrue(jmp) | Card::Jump(jmp) => {
                self._encode_jump(nodeid, &jmp)?;
            }
            Card::StringLiteral(c) => self._push_data(c.0)?,
            Card::Call(c) => {
                let name = &c.0;
                let key = Key::from_str(name.as_str()).unwrap();
                key.encode(&mut self.program.bytecode).unwrap();
            }
            Card::ScalarArray(n) => {
                n.0.encode(&mut self.program.bytecode).unwrap();
            }
            Card::ExitWithCode(s) => {
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                s.0.encode(&mut self.program.bytecode).unwrap();
                self.program.bytecode.push(Instruction::Exit as u8);
            }
            Card::ScalarLabel(s) | Card::ScalarInt(s) => {
                s.0.encode(&mut self.program.bytecode).unwrap();
            }
            Card::ScalarFloat(s) => {
                s.0.encode(&mut self.program.bytecode).unwrap();
            }
            Card::ScalarNull
            | Card::Return
            | Card::And
            | Card::Not
            | Card::Or
            | Card::Xor
            | Card::Pop
            | Card::Equals
            | Card::Less
            | Card::LessOrEq
            | Card::NotEquals
            | Card::Exit
            | Card::Pass
            | Card::CopyLast
            | Card::Add
            | Card::Sub
            | Card::Mul
            | Card::Div
            | Card::ClearStack => { /* These cards translate to a single instruction */ }
        }
        Ok(())
    }
}
