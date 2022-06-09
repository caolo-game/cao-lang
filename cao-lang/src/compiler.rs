//! The compiler module that transforms [CaoIr](CaoIr) into bytecode.
//!
mod card;
mod compilation_error;
mod compile_options;
mod lane;
mod program;

pub mod card_description;

mod compiled_lane;
#[cfg(test)]
mod tests;

use crate::{
    bytecode::{encode_str, write_to_vec},
    collections::key_map::{Handle, KeyMap},
    compiled_program::{CaoCompiledProgram, Label},
    prelude::TraceEntry,
    Instruction, NodeId, VariableId,
};
use std::fmt::Debug;
use std::mem;
use std::{cell::RefCell, convert::TryFrom};
use std::{convert::TryInto, str::FromStr};

pub use card::*;
pub use compilation_error::*;
pub use compile_options::*;
pub use lane::*;
pub use program::*;

use self::compiled_lane::CompiledLane;

pub type CompilationResult<T> = Result<T, CompilationError>;

/// Intermediate representation of a Cao-Lang program.
///
/// Execution will begin with the first Lane
pub(crate) type CaoIr<'a> = &'a [CompiledLane];
pub(crate) type NameSpace = smallvec::SmallVec<[String; 8]>;

pub struct Compiler<'a> {
    options: CompileOptions,
    program: CaoCompiledProgram,
    next_var: RefCell<VariableId>,

    /// maps lanes to their metadata
    jump_table: KeyMap<LaneMeta>,

    current_namespace: NameSpace,
    locals: Box<arrayvec::ArrayVec<Local<'a>, 255>>,
    scope_depth: i32,
    current_card: i32,
    current_lane: String,
}

#[derive(Debug, Clone, Copy)]
struct LaneMeta {
    pub hash_key: Handle,
    /// number of arguments
    pub arity: u32,
}

/// local variables during compilation
#[derive(Debug)]
pub(crate) struct Local<'a> {
    pub name: &'a str,
    pub depth: i32,
}

pub fn compile(
    compilation_unit: CaoProgram,
    compile_options: impl Into<Option<CompileOptions>>,
) -> CompilationResult<CaoCompiledProgram> {
    let compilation_unit = compilation_unit
        .into_ir_stream()
        .map_err(|err| CompilationError::with_loc(err, LaneNode::default(), 0))?;

    let mut compiler = Compiler::new();
    compiler.compile(&compilation_unit, compile_options)
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            options: Default::default(),
            program: CaoCompiledProgram::default(),
            next_var: RefCell::new(VariableId(0)),
            jump_table: Default::default(),
            current_namespace: Default::default(),
            locals: Default::default(),
            scope_depth: 0,
            current_card: -1,
            current_lane: "".to_owned(),
        }
    }

    pub fn compile(
        &mut self,
        compilation_unit: CaoIr<'a>,
        compile_options: impl Into<Option<CompileOptions>>,
    ) -> CompilationResult<CaoCompiledProgram> {
        self.options = compile_options.into().unwrap_or_default();
        // minimize the surface of the generic function
        self._compile(compilation_unit)
    }

    fn _compile(&mut self, compilation_unit: CaoIr<'a>) -> CompilationResult<CaoCompiledProgram> {
        if compilation_unit.is_empty() {
            return Err(CompilationError::with_loc(
                CompilationErrorPayload::EmptyProgram,
                LaneNode::default(),
                0,
            ));
        }
        // initialize
        self.program = CaoCompiledProgram::default();
        self.next_var = RefCell::new(VariableId(0));
        self.compile_stage_1(compilation_unit)?;
        self.compile_stage_2(compilation_unit)?;

        Ok(mem::take(&mut self.program))
    }

    fn error(&self, pl: CompilationErrorPayload) -> CompilationError {
        CompilationError::with_loc(
            pl,
            LaneNode(self.current_lane.to_string()),
            self.current_card,
        )
    }

    /// build the jump table and consume the lane names
    /// also reserve memory for the program labels
    fn compile_stage_1(&mut self, compilation_unit: CaoIr) -> CompilationResult<()> {
        // check if len fits in 16 bits
        let _: u16 = match compilation_unit.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(self.error(CompilationErrorPayload::TooManyLanes)),
        };

        let mut num_cards = 0usize;
        self.current_card = -1;
        for (i, n) in compilation_unit.iter().enumerate() {
            self.current_lane = n.name.clone();
            num_cards += n.cards.len();

            let nodekey = Handle::from_u64(
                NodeId {
                    lane: i
                        .try_into()
                        .map_err(|_| self.error(CompilationErrorPayload::TooManyLanes))?,
                    pos: 0,
                }
                .into(),
            );
            self.add_lane(nodekey, n)?;
        }

        self.program.labels.0.reserve(num_cards).expect("reserve");
        Ok(())
    }

    fn add_lane(&mut self, nodekey: Handle, n: &CompiledLane) -> CompilationResult<()> {
        let metadata = LaneMeta {
            hash_key: nodekey,
            arity: n.arguments.len() as u32,
        };
        let namekey = Handle::from_str(n.name.as_str()).expect("Failed to hash lane name");
        if self.jump_table.contains(namekey) {
            return Err(self.error(CompilationErrorPayload::DuplicateName(n.name.clone())));
        }
        self.jump_table.insert(namekey, metadata).unwrap();
        Ok(())
    }

    /// consume lane cards and build the bytecode
    fn compile_stage_2(&mut self, compilation_unit: CaoIr<'a>) -> CompilationResult<()> {
        let mut lanes = compilation_unit.iter().enumerate();

        if let Some((il, main_lane)) = lanes.next() {
            let len: u32 = match main_lane.cards.len().try_into() {
                Ok(i) => i,
                Err(_) => return Err(self.error(CompilationErrorPayload::TooManyCards(il))),
            };
            self.scope_begin();
            self.process_lane(il, main_lane, 0)?;
            let nodeid = NodeId {
                lane: il as u32,
                pos: len,
            };
            self.scope_end();
            // insert explicit exit after the first lane
            self.process_card(nodeid, &Card::Abort)?;
        }

        for (il, lane) in lanes {
            let nodeid = NodeId {
                lane: il as u32,
                pos: 0,
            };
            let nodeid_hash = Handle::from_u64(nodeid.into());
            let handle = u32::try_from(self.program.bytecode.len())
                .expect("bytecode length to fit into 32 bits");
            self.program
                .labels
                .0
                .insert(nodeid_hash, Label::new(handle))
                .unwrap();

            self.scope_begin();

            self.process_lane(il, lane, 1)?;

            self.scope_end();
            self.push_instruction(Instruction::Return);
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
            self.push_instruction(Instruction::Pop);
        }
    }

    /// add a local variable
    ///
    /// return its index
    fn add_local(&mut self, name: &'a str) -> CompilationResult<u32> {
        self.validate_var_name(name)?;
        let result = self.locals.len();
        self.locals
            .try_push(Local {
                name,
                depth: self.scope_depth,
            })
            .map_err(|_| self.error(CompilationErrorPayload::TooManyLocals))?;
        Ok(result as u32)
    }

    fn process_lane(
        &mut self,
        il: usize,
        CompiledLane {
            name,
            arguments,
            cards,
            namespace,
        }: &'a CompiledLane,
        instruction_offset: i32,
    ) -> CompilationResult<()> {
        self.current_lane = name.clone();
        self.current_card = -1;
        self.current_namespace = namespace.clone();

        // check if len fits in 16 bits
        let _len: u16 = match cards.len().try_into() {
            Ok(i) => i,
            Err(_) => return Err(self.error(CompilationErrorPayload::TooManyCards(il))),
        };
        // at runtime: pop arguments in the same order as the variables were declared
        for param in arguments.iter() {
            self.add_local(param.as_str())?;
        }
        for (ic, card) in cards.iter().enumerate() {
            self.current_card = ic as i32;
            let nodeid = NodeId {
                lane: il as u32,
                pos: (ic as i32 + instruction_offset) as u32,
            };
            self.process_card(nodeid, card)?;
        }
        Ok(())
    }

    /// encodes the then block (card), and conditionally jumps over it using the `skip_instr`
    fn encode_if_then(
        &mut self,
        skip_instr: Instruction,
        then: impl FnOnce(&mut Self) -> CompilationResult<()>,
    ) -> CompilationResult<()> {
        type Pos = i32;
        assert!(
            matches!(
                skip_instr,
                Instruction::GotoIfTrue | Instruction::GotoIfFalse
            ),
            "invalid skip instruction"
        );
        self.push_instruction(skip_instr);
        let idx = self.program.bytecode.len();
        write_to_vec(0 as Pos, &mut self.program.bytecode);

        // write the `then` block
        then(self)?;

        unsafe {
            let ptr = self.program.bytecode.as_mut_ptr().add(idx) as *mut Pos;
            std::ptr::write(ptr, self.program.bytecode.len() as Pos);
        };
        Ok(())
    }

    fn encode_jump(&mut self, lane: &LaneNode) -> CompilationResult<()> {
        let mut to = self.jump_table.get(Handle::from(lane));
        let mut i = self.current_namespace.len();
        while i > 0 && to.is_none() {
            // attempt to look up the function in the current namespace
            //
            // TODO: create handle from parts instead of building the string..
            let mut name = self.current_namespace[0..i].join(".");
            name.push('.');
            name.push_str(lane.0.as_str());
            let handle = Handle::from(name.as_str());
            to = self.jump_table.get(handle);
            i -= 1;
        }
        let to = to.ok_or_else(|| {
            self.error(CompilationErrorPayload::InvalidJump {
                dst: lane.clone(),
                msg: None,
            })
        })?;
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
            if local.name == name {
                return Ok(i as isize);
            }
        }
        Ok(-1)
    }

    fn process_card(&mut self, nodeid: NodeId, card: &'a Card) -> CompilationResult<()> {
        let handle = u32::try_from(self.program.bytecode.len())
            .expect("bytecode length to fit into 32 bits");
        let nodeid_hash = Handle::from_u64(nodeid.into());
        self.program
            .labels
            .0
            .insert(nodeid_hash, Label::new(handle))
            .unwrap();

        if let Some(instr) = card.instruction() {
            // instruction itself
            self.push_instruction(instr);
        }
        match card {
            Card::Noop => {}
            Card::CompositeCard { cards, .. } => {
                for card in cards.iter() {
                    self.process_card(nodeid, card)?;
                }
            }
            Card::ForEach { variable, lane } => {
                let target_lane = Handle::from(lane);
                let arity = self.jump_table[target_lane].arity;
                if arity != 2 {
                    return Err(self.error(CompilationErrorPayload::InvalidJump {
                        dst: lane.clone(),
                        msg: Some("ForEach lanes need to have 2 parameters".to_string()),
                    }));
                }
                self.read_var_card(variable)?;
                self.push_instruction(Instruction::BeginForEach);
                let block_begin = self.program.bytecode.len() as i32;
                self.push_instruction(Instruction::ForEach);
                self.encode_jump(lane)?;
                // return to the repeat instruction
                self.push_instruction(Instruction::GotoIfTrue);
                write_to_vec(block_begin, &mut self.program.bytecode);
            }
            // TODO: blocked by lane ABI
            Card::While(_) => {
                return Err(self.error(CompilationErrorPayload::Unimplemented("While cards")))
            }
            Card::Repeat(repeat) => {
                let target_lane = Handle::from(repeat);
                let arity = self.jump_table[target_lane].arity;
                if arity != 1 {
                    return Err(self.error(CompilationErrorPayload::InvalidJump {
                        dst: repeat.clone(),
                        msg: Some("Repeat lanes need to have 1 parameter".to_string()),
                    }));
                }
                self.push_instruction(Instruction::BeginRepeat);
                let block_begin = self.program.bytecode.len() as i32;
                self.push_instruction(Instruction::Repeat);
                self.encode_jump(repeat)?;
                // return to the repeat instruction
                self.push_instruction(Instruction::GotoIfTrue);
                write_to_vec(block_begin, &mut self.program.bytecode);
            }
            Card::ReadVar(variable) => {
                self.read_var_card(variable)?;
            }
            Card::SetVar(var) => {
                let index = self.add_local(&*var.0)?;
                self.push_instruction(Instruction::SetLocalVar);
                write_to_vec(index, &mut self.program.bytecode);
            }
            Card::SetGlobalVar(variable) => {
                let mut next_var = self.next_var.borrow_mut();
                if variable.0.is_empty() {
                    return Err(self.error(CompilationErrorPayload::EmptyVariable));
                }
                let varhash = Handle::from_bytes(variable.0.as_bytes());

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
                    .entry(Handle::from_u32(id.0))
                    .or_insert_with(move || *variable.0);
                write_to_vec(*id, &mut self.program.bytecode);
            }
            Card::IfElse {
                then: then_card,
                r#else: else_card,
            } => {
                let mut idx = 0;
                self.encode_if_then(Instruction::GotoIfFalse, |c| {
                    c.process_card(nodeid, then_card)?;
                    // jump over the `else` branch
                    c.push_instruction(Instruction::Goto);
                    idx = c.program.bytecode.len();
                    write_to_vec(0i32, &mut c.program.bytecode);
                    Ok(())
                })?;
                self.process_card(nodeid, else_card)?;
                unsafe {
                    let ptr = self.program.bytecode.as_mut_ptr().add(idx) as *mut i32;
                    std::ptr::write(ptr, self.program.bytecode.len() as i32);
                }
            }
            Card::IfFalse(jmp) => {
                self.encode_if_then(Instruction::GotoIfTrue, |c| c.process_card(nodeid, jmp))?
            }
            Card::IfTrue(jmp) => {
                self.encode_if_then(Instruction::GotoIfFalse, |c| c.process_card(nodeid, jmp))?
            }
            Card::Jump(jmp) => self.encode_jump(jmp)?,
            Card::StringLiteral(c) => self.push_str(c.0.as_str()),
            Card::CallNative(c) => {
                let name = &c.0;
                let key = Handle::from_str(name.as_str()).unwrap();
                write_to_vec(key, &mut self.program.bytecode);
            }
            Card::ScalarInt(s) => {
                write_to_vec(s.0, &mut self.program.bytecode);
            }
            Card::ScalarFloat(s) => {
                write_to_vec(s.0, &mut self.program.bytecode);
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
            | Card::Len
            | Card::GetProperty
            | Card::SetProperty
            | Card::ClearStack => { /* These cards translate to a single instruction */ }
        }
        Ok(())
    }

    fn read_var_card(&mut self, variable: &VarNode) -> CompilationResult<()> {
        let scope = self.resolve_var(variable.0.as_str())?;
        if scope < 0 {
            // global
            let mut next_var = self.next_var.borrow_mut();
            let varhash = Handle::from_bytes(variable.0.as_bytes());
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
            let id = *id;
            self.program
                .variables
                .names
                .entry(Handle::from_u32(id.0))
                .or_insert_with(|| *variable.0);
            self.push_instruction(Instruction::ReadGlobalVar);
            write_to_vec(id, &mut self.program.bytecode);
        } else {
            //local
            self.read_local_var(scope as u32);
        }
        Ok(())
    }

    fn read_local_var(&mut self, index: u32) {
        self.push_instruction(Instruction::ReadLocalVar);
        write_to_vec(index, &mut self.program.bytecode);
    }

    fn validate_var_name(&self, name: &str) -> CompilationResult<()> {
        if name.is_empty() {
            return Err(self.error(CompilationErrorPayload::EmptyVariable));
        }
        Ok(())
    }

    fn push_instruction(&mut self, instruction: Instruction) {
        self.program.trace.insert(
            self.program.bytecode.len(),
            TraceEntry {
                lane: self.current_lane.clone(),
                card: self.current_card,
            },
        );
        self.program.bytecode.push(instruction as u8);
    }
}
