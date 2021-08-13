mod card;
mod compilation_error;
mod compile_options;
mod lane;

pub mod card_description;

#[cfg(test)]
mod tests;

use crate::{
    bytecode::{encode_str, write_to_vec},
    collections::key_map::{Key, KeyMap},
    program::{CaoProgram, Label},
    Instruction, VarName,
};
use crate::{NodeId, VariableId};
use std::fmt::Debug;
use std::mem;
use std::{cell::RefCell, convert::TryFrom};
use std::{convert::TryInto, str::FromStr};

pub use card::*;
pub use compilation_error::*;
pub use compile_options::*;
pub use lane::*;

pub type CompilationResult<T> = Result<T, CompilationError>;

/// Intermediate representation of a Cao-Lang program.
///
/// Execution will begin with the first Lane
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CaoIr {
    pub lanes: Vec<Lane>,
}

pub struct Compiler<'a> {
    pub(crate) options: CompileOptions,
    pub(crate) program: CaoProgram,
    next_var: RefCell<VariableId>,

    /// maps lanes to their pre-hash-map keys
    jump_table: KeyMap<LaneMeta>,

    locals: Box<arrayvec::ArrayVec<Local<'a>, 255>>,
    scope_depth: i32,
    current_card: i32,
    current_lane: LaneNode,

    _m: std::marker::PhantomData<&'a ()>,
}

#[derive(Debug, Clone, Copy)]
struct LaneMeta {
    pub hash_key: Key,
    /// number of arguments
    pub arity: u32,
}

/// local variables during compilation
#[derive(Debug)]
pub struct Local<'a> {
    pub name: VarName,
    pub depth: i32,
    _m: std::marker::PhantomData<&'a ()>,
}

pub fn compile(
    compilation_unit: CaoIr,
    compile_options: impl Into<Option<CompileOptions>>,
) -> CompilationResult<CaoProgram> {
    let mut compiler = Compiler::new();
    compiler.compile(compilation_unit, compile_options)
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            program: CaoProgram::default(),
            jump_table: Default::default(),
            options: Default::default(),
            next_var: RefCell::new(VariableId(0)),
            locals: Default::default(),
            scope_depth: 0,
            current_lane: LaneNode::LaneId(0),
            current_card: -1,
            _m: Default::default(),
        }
    }

    pub fn compile(
        &mut self,
        compilation_unit: CaoIr,
        compile_options: impl Into<Option<CompileOptions>>,
    ) -> CompilationResult<CaoProgram> {
        self.options = compile_options.into().unwrap_or_default();
        // minimize the surface of the generic function
        self._compile(compilation_unit)
    }

    fn _compile(&mut self, mut compilation_unit: CaoIr) -> CompilationResult<CaoProgram> {
        if compilation_unit.lanes.is_empty() {
            return Err(CompilationError::with_loc(
                CompilationErrorPayload::EmptyProgram,
                LaneNode::LaneId(0),
                0,
            ));
        }
        // initialize
        self.program = CaoProgram::default();
        self.next_var = RefCell::new(VariableId(0));
        self.compile_stage_1(&mut compilation_unit)?;
        self.compile_stage_2(compilation_unit)?;

        Ok(mem::take(&mut self.program))
    }

    fn error(&self, pl: CompilationErrorPayload) -> CompilationError {
        CompilationError::with_loc(pl, self.current_lane.clone(), self.current_card)
    }

    /// build the jump table and consume the lane names
    /// also reserve memory for the program labels
    fn compile_stage_1(&mut self, compilation_unit: &mut CaoIr) -> CompilationResult<()> {
        // check if len fits in 16 bits
        let _: u16 = match compilation_unit.lanes.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(self.error(CompilationErrorPayload::TooManyLanes)),
        };

        let mut num_cards = 0usize;
        self.current_card = -1;
        for (i, n) in compilation_unit.lanes.iter_mut().enumerate() {
            self.current_lane = LaneNode::LaneId(i);

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
            let metadata = LaneMeta {
                hash_key: nodekey,
                arity: n.arguments.len() as u32,
            };
            self.jump_table.insert(indexkey, metadata).unwrap();
            if let Some(ref mut name) = n.name.as_mut() {
                let namekey = Key::from_str(name.as_str()).expect("Failed to hash lane name");
                if self.jump_table.contains(namekey) {
                    return Err(
                        self.error(CompilationErrorPayload::DuplicateName(std::mem::take(name)))
                    );
                }
                self.jump_table.insert(namekey, metadata).unwrap();
            }
        }

        self.program.labels.0.reserve(num_cards).expect("reserve");
        Ok(())
    }

    /// consume lane cards and build the bytecode
    fn compile_stage_2(&mut self, compilation_unit: CaoIr) -> CompilationResult<()> {
        let mut lanes = compilation_unit.lanes.into_iter().enumerate();

        if let Some((il, main_lane)) = lanes.next() {
            let len: u16 = match main_lane.cards.len().try_into() {
                Ok(i) => i,
                Err(_) => return Err(self.error(CompilationErrorPayload::TooManyCards(il))),
            };
            self.scope_begin();
            self.process_lane(il, main_lane, 0)?;
            let nodeid = NodeId {
                lane: il as u16,
                pos: len,
            };
            self.scope_end();
            // insert explicit exit after the first lane
            self.process_card(nodeid, Card::Abort)?;
        }

        for (il, lane) in lanes {
            {
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
                    .insert(nodeid_hash, Label::new(handle))
                    .unwrap();
            }

            self.scope_begin();

            // process the lane
            self.process_lane(il, lane, 1)?;

            self.scope_end();
            self.program.bytecode.push(Instruction::Return as u8);
        }

        Ok(())
    }

    fn scope_begin(&mut self) {
        self.scope_depth += 1;
    }

    fn scope_end(&mut self) {
        self.scope_depth -= 1;
        // while the last item's depth is greater than scope_depth
        while self
            .locals
            .last()
            .map(|l| l.depth > self.scope_depth)
            .unwrap_or(false)
        {
            self.locals.pop();
            // we can clean up a bit.
            // Note that this might leave garbage values on the stack,
            // but the VM clears those on Returns.
            self.program.bytecode.push(Instruction::Pop as u8);
        }
    }

    fn add_local(&mut self, name: VarName) -> CompilationResult<()> {
        self.validate_var_name(&name)?;
        self.locals
            .try_push(Local {
                name,
                depth: self.scope_depth,
                _m: Default::default(),
            })
            .map_err(|_| self.error(CompilationErrorPayload::TooManyLocals))?;
        Ok(())
    }

    fn process_lane(
        &mut self,
        il: usize,
        Lane {
            cards, arguments, ..
        }: Lane,
        // cards: Vec<Card>,
        instruction_offset: i32,
    ) -> CompilationResult<()> {
        self.current_lane = LaneNode::LaneId(il);
        self.current_card = -1;

        // check if len fits in 16 bits
        let _len: u16 = match cards.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(self.error(CompilationErrorPayload::TooManyCards(il))),
        };
        // at runtime: pop arguments in the same order as the variables were declared
        for param in arguments.iter() {
            self.add_local(VarName::from_str(param.as_str()).map_err(|_| {
                self.error(CompilationErrorPayload::BadVariableName(param.to_string()))
            })?)?;
        }
        for (ic, card) in cards.into_iter().enumerate() {
            self.current_card = ic as i32;
            let nodeid = NodeId {
                lane: il as u16,
                pos: (ic as i32 + instruction_offset) as u16,
            };
            self.process_card(nodeid, card)?;
        }
        Ok(())
    }

    fn conditional_jump(
        &mut self,
        skip_instr: Instruction,
        nodeid: NodeId,
        lane: &LaneNode,
    ) -> CompilationResult<()> {
        assert!(
            matches!(
                skip_instr,
                Instruction::GotoIfTrue | Instruction::GotoIfFalse
            ),
            "invalid skip instruction"
        );
        self.program.bytecode.push(skip_instr as u8);
        let pos = instruction_span(Instruction::CallLane) + self.program.bytecode.len() as i32 + 4; // +4 == sizeof pos
        debug_assert_eq!(std::mem::size_of_val(&pos), 4);
        write_to_vec(pos, &mut self.program.bytecode);
        self.program.bytecode.push(Instruction::CallLane as u8);
        self.encode_jump(nodeid, lane)?;
        Ok(())
    }

    fn encode_jump(&mut self, nodeid: NodeId, lane: &LaneNode) -> CompilationResult<()> {
        let to = match lane {
            LaneNode::LaneName(lane) => self
                .jump_table
                .get(Key::from_str(lane).expect("Failed to hash jump target name"))
                .ok_or_else(|| {
                    self.error(CompilationErrorPayload::InvalidJump {
                        src: nodeid,
                        dst: lane.to_string(),
                        msg: None,
                    })
                })?,
            LaneNode::LaneId(id) => {
                self.jump_table
                    .get(Key::from_u32(*id as u32))
                    .ok_or_else(|| {
                        self.error(CompilationErrorPayload::InvalidJump {
                            src: nodeid,
                            dst: format!("Lane id {}", id),
                            msg: None,
                        })
                    })?
            }
        };
        write_to_vec(to.hash_key, &mut self.program.bytecode);
        write_to_vec(to.arity, &mut self.program.bytecode);
        Ok(())
    }

    fn push_str(&mut self, data: &str) {
        let handle = self.program.data.len() as u32;
        write_to_vec(handle, &mut self.program.bytecode);

        encode_str(data, &mut self.program.data);
    }

    fn resolve_var(&self, name: &str) -> CompilationResult<isize> {
        self.validate_var_name(name)?;
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name.as_str() == name {
                return Ok(i as isize);
            }
        }
        Ok(-1)
    }

    fn process_card(&mut self, nodeid: NodeId, card: Card) -> CompilationResult<()> {
        let handle = u32::try_from(self.program.bytecode.len())
            .expect("bytecode length to fit into 32 bits");
        let nodeid_hash = Key::from_u32(nodeid.into());
        self.program
            .labels
            .0
            .insert(nodeid_hash, Label::new(handle))
            .unwrap();

        if let Some(instr) = card.instruction() {
            // instruction itself
            self.program.bytecode.push(instr as u8);
        }
        match card {
            // TODO: blocked by lane ABI
            Card::While(_) => {
                return Err(self.error(CompilationErrorPayload::Unimplemented("While cards")))
            }
            Card::Repeat(repeat) => {
                // Init, add 1
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                write_to_vec(1i64, &mut self.program.bytecode);
                self.program.bytecode.push(Instruction::Add as u8);
                // Condition
                let cond_block_begin = self.program.bytecode.len() as i32;
                self.program.bytecode.push(Instruction::ScalarInt as u8);
                write_to_vec(1i64, &mut self.program.bytecode);
                self.program.bytecode.push(Instruction::Sub as u8);
                self.program.bytecode.push(Instruction::CopyLast as u8);
                self.program.bytecode.push(Instruction::GotoIfFalse as u8);
                let execute_block_len =
                    instruction_span(Instruction::CallLane) + instruction_span(Instruction::Goto);
                write_to_vec(
                    self.program.bytecode.len() as i32 + 4 + execute_block_len,
                    &mut self.program.bytecode,
                );
                // Execute
                self.program.bytecode.push(Instruction::CallLane as u8);
                self.encode_jump(nodeid, &repeat)?;
                self.program.bytecode.push(Instruction::Goto as u8);
                write_to_vec(cond_block_begin, &mut self.program.bytecode);
            }
            Card::ReadVar(variable) => {
                let scope = self.resolve_var(variable.0.as_str())?;
                if scope < 0 {
                    // global
                    let mut next_var = self.next_var.borrow_mut();
                    let varhash = Key::from_bytes(variable.0.as_bytes());
                    let id = self
                        .program
                        .variables
                        .ids
                        .entry(varhash)
                        .or_insert_with(move || {
                            let id = *next_var;
                            *next_var = VariableId(id.0 + 1);
                            id
                        });
                    self.program
                        .variables
                        .names
                        .entry(*id)
                        .or_insert_with(|| variable.0);
                    self.program.bytecode.push(Instruction::ReadGlobalVar as u8);
                    write_to_vec(*id, &mut self.program.bytecode);
                } else {
                    //local
                    self.program.bytecode.push(Instruction::ReadLocalVar as u8);
                    let index = scope as u32;
                    write_to_vec(index, &mut self.program.bytecode);
                }
            }
            Card::SetVar(var) => {
                let index = self.locals.len() as u32;
                self.add_local(var.0)?;
                self.program.bytecode.push(Instruction::SetLocalVar as u8);
                write_to_vec(index, &mut self.program.bytecode);
            }
            Card::SetGlobalVar(variable) => {
                let mut next_var = self.next_var.borrow_mut();
                if variable.0.is_empty() {
                    return Err(self.error(CompilationErrorPayload::EmptyVariable));
                }
                let varhash = Key::from_bytes(variable.0.as_bytes());

                let id = self
                    .program
                    .variables
                    .ids
                    .entry(varhash)
                    .or_insert_with(move || {
                        let id = *next_var;
                        *next_var = VariableId(id.0 + 1);
                        id
                    });
                self.program
                    .variables
                    .names
                    .entry(*id)
                    .or_insert_with(move || variable.0);
                write_to_vec(*id, &mut self.program.bytecode);
            }
            Card::IfElse {
                then: then_lane,
                r#else: else_lane,
            } => {
                // if true jump to then (2nd item) else execute 1st item then jump over the 2nd
                self.program.bytecode.push(Instruction::GotoIfTrue as u8);
                let pos = instruction_span(Instruction::Goto)
                    + instruction_span(Instruction::CallLane)
                    + self.program.bytecode.len() as i32
                    + 4; // +4 == sizeof pos
                debug_assert_eq!(std::mem::size_of_val(&pos), 4);
                write_to_vec(pos, &mut self.program.bytecode);
                // else
                self.program.bytecode.push(Instruction::CallLane as u8);
                self.encode_jump(nodeid, &else_lane)?;

                self.program.bytecode.push(Instruction::Goto as u8);
                let pos = instruction_span(Instruction::CallLane)
                    + self.program.bytecode.len() as i32
                    + 4; // +4 == sizeof pos
                write_to_vec(pos, &mut self.program.bytecode);
                // then
                self.program.bytecode.push(Instruction::CallLane as u8);
                self.encode_jump(nodeid, &then_lane)?;
            }
            Card::IfFalse(jmp) => {
                // if the value is true we DON'T jump
                self.conditional_jump(Instruction::GotoIfTrue, nodeid, &jmp)?;
            }
            Card::IfTrue(jmp) => {
                // if the value is false we DON'T jump
                self.conditional_jump(Instruction::GotoIfFalse, nodeid, &jmp)?;
            }
            Card::Jump(jmp) => {
                self.encode_jump(nodeid, &jmp)?;
            }
            Card::StringLiteral(c) => self.push_str(c.0.as_str()),
            Card::CallNative(c) => {
                let name = &c.0;
                let key = Key::from_str(name.as_str()).unwrap();
                write_to_vec(key, &mut self.program.bytecode);
            }
            Card::ScalarInt(s) => {
                write_to_vec(s.0, &mut self.program.bytecode);
            }
            Card::ScalarFloat(s) => {
                write_to_vec(s.0, &mut self.program.bytecode);
            }
            Card::GetProperty(VarNode(name)) | Card::SetProperty(VarNode(name)) => {
                self.validate_var_name(name.as_str())?;
                let handle = Key::from_str(name.as_str()).unwrap();
                write_to_vec(handle, &mut self.program.bytecode);
            }
            Card::ScalarNil
            | Card::Return
            | Card::And
            | Card::Abort
            | Card::Not
            | Card::Or
            | Card::Xor
            | Card::Pop
            | Card::Equals
            | Card::Less
            | Card::LessOrEq
            | Card::NotEquals
            | Card::Pass
            | Card::CopyLast
            | Card::Add
            | Card::Sub
            | Card::Mul
            | Card::Div
            | Card::CreateTable
            | Card::ClearStack => { /* These cards translate to a single instruction */ }
        }
        Ok(())
    }

    fn validate_var_name(&self, name: &str) -> CompilationResult<()> {
        if name.is_empty() {
            return Err(self.error(CompilationErrorPayload::EmptyVariable));
        }
        Ok(())
    }
}

/// return the number of bytes this instruction spans in the bytecode
const fn instruction_span(instr: Instruction) -> i32 {
    match instr {
        Instruction::Add
        | Instruction::Sub
        | Instruction::Exit
        | Instruction::Mul
        | Instruction::Div
        | Instruction::Call
        | Instruction::Equals
        | Instruction::NotEquals
        | Instruction::Less
        | Instruction::LessOrEq
        | Instruction::Pop
        | Instruction::Pass
        | Instruction::ScalarNil
        | Instruction::ClearStack
        | Instruction::CopyLast
        | Instruction::Return
        | Instruction::SwapLast
        | Instruction::And
        | Instruction::Or
        | Instruction::Xor
        | Instruction::InitTable
        | Instruction::Not => 1,
        //
        Instruction::ScalarInt | Instruction::ScalarFloat => 9,
        Instruction::StringLiteral => 5,
        //
        Instruction::SetLocalVar
        | Instruction::ReadLocalVar
        | Instruction::SetProperty
        | Instruction::GetProperty
        | Instruction::SetGlobalVar
        | Instruction::ReadGlobalVar => 5,
        //
        Instruction::Goto | Instruction::GotoIfTrue | Instruction::GotoIfFalse => 5,
        Instruction::CallLane => 9,
    }
}
