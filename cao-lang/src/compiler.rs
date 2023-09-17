//! The compiler module that transforms [CaoIr](CaoIr) into bytecode.
//!
mod card;
mod compilation_error;
mod compile_options;
mod function;
mod module;

mod function_ir;
#[cfg(test)]
mod tests;

use crate::{
    bytecode::{encode_str, write_to_vec},
    collections::{handle_table::Handle, hash_map::CaoHashMap},
    compiled_program::{CaoCompiledProgram, Label},
    prelude::Trace,
    Instruction, VariableId,
};
use core::slice;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::mem;
use std::{convert::TryInto, str::FromStr};

pub use card::*;
pub use compilation_error::*;
pub use compile_options::*;
pub use function::*;
pub use module::*;

use self::function_ir::FunctionIr;

pub type CompilationResult<T> = Result<T, CompilationError>;

/// Intermediate representation of a Cao-Lang program.
///
/// Execution will begin with the first Function
pub(crate) type FunctionSlice<'a> = &'a [FunctionIr];
pub(crate) type NameSpace = smallvec::SmallVec<[Box<str>; 8]>;
pub(crate) type ImportsIr = std::collections::HashMap<String, String>;
pub(crate) type Locals<'a> = arrayvec::ArrayVec<Local<'a>, 255>;
type Upvalues = arrayvec::ArrayVec<Upvalue, 255>;

pub struct Compiler<'a> {
    options: CompileOptions,
    program: CaoCompiledProgram,
    next_var: VariableId,

    /// maps functions to their metadata
    jump_table: CaoHashMap<String, FunctionMeta>,

    current_namespace: Cow<'a, NameSpace>,
    current_imports: Cow<'a, ImportsIr>,
    locals: Vec<Locals<'a>>,
    upvalues: Vec<Upvalues>,
    scope_depth: Vec<i32>,
    current_index: CardIndex,
    function_id: usize,
}

#[derive(Debug, Clone, Copy)]
struct FunctionMeta {
    pub hash_key: Handle,
    /// number of arguments
    pub arity: u32,
}

/// local variables during compilation
#[derive(Debug)]
pub(crate) struct Local<'a> {
    pub name: &'a str,
    pub depth: i32,
    pub captured: bool,
}

pub fn compile(
    compilation_unit: CaoProgram,
    compile_options: impl Into<Option<CompileOptions>>,
) -> CompilationResult<CaoCompiledProgram> {
    let options = compile_options.into().unwrap_or_default();
    let compilation_unit = compilation_unit
        .into_ir_stream(options.recursion_limit)
        .map_err(|err| CompilationError::with_loc(err, Trace::default()))?;

    let mut compiler = Compiler::new();
    compiler.compile(&compilation_unit, options)
}

impl<'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

enum Variable {
    Global,
    Local(usize),
    /// Upvalues are captured variables from enclosing scopes
    Upvalue(usize),
}

#[derive(Default, Clone, Copy, Debug)]
struct Upvalue {
    is_local: bool,
    index: u8,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            options: Default::default(),
            program: CaoCompiledProgram::default(),
            next_var: VariableId(0),
            jump_table: Default::default(),
            current_namespace: Default::default(),
            locals: vec![Default::default()],
            upvalues: vec![Default::default()],
            scope_depth: vec![0],
            current_index: CardIndex::default(),
            current_imports: Default::default(),
            function_id: 0,
        }
    }

    fn trace(&self) -> Trace {
        Trace {
            namespace: self.current_namespace.to_owned().into_owned(),
            index: self.current_index.clone(),
        }
    }

    pub fn compile(
        &mut self,
        compilation_unit: FunctionSlice<'a>,
        compile_options: CompileOptions,
    ) -> CompilationResult<CaoCompiledProgram> {
        self.options = compile_options;
        if compilation_unit.is_empty() {
            return Err(CompilationError::with_loc(
                CompilationErrorPayload::EmptyProgram,
                self.trace(),
            ));
        }
        self.program = CaoCompiledProgram::default();
        self.next_var = VariableId(0);
        self.compile_stage_1(compilation_unit)?;
        self.compile_stage_2(compilation_unit)?;

        self.current_imports = Default::default();
        // the last instruction is a trap for native to cao-lang function calls
        self.push_instruction(Instruction::Exit);
        Ok(mem::take(&mut self.program))
    }

    fn error(&self, pl: CompilationErrorPayload) -> CompilationError {
        CompilationError::with_loc(pl, self.trace())
    }

    /// build the jump table and consume the function names
    fn compile_stage_1(&mut self, compilation_unit: FunctionSlice) -> CompilationResult<()> {
        let mut num_cards = 0usize;
        for (i, n) in compilation_unit.iter().enumerate() {
            self.current_index.function = i;
            self.current_index.card_index.indices.clear();
            num_cards += n.cards.len();

            let handle = self.current_index.as_handle();
            self.add_function(handle, n)?;
        }

        self.program.labels.0.reserve(num_cards).expect("reserve");
        Ok(())
    }

    fn add_function(&mut self, handle: Handle, n: &FunctionIr) -> CompilationResult<()> {
        let metadata = FunctionMeta {
            hash_key: handle,
            arity: n.arguments.len() as u32,
        };
        if self.jump_table.contains(n.name.as_ref()) {
            return Err(self.error(CompilationErrorPayload::DuplicateName(n.name.to_string())));
        }
        self.jump_table
            .insert(n.name.to_string(), metadata)
            .unwrap();
        Ok(())
    }

    /// consume function cards and build the bytecode
    fn compile_stage_2(&mut self, compilation_unit: FunctionSlice<'a>) -> CompilationResult<()> {
        let mut functions = compilation_unit.iter().enumerate();

        if let Some((il, main_function)) = functions.next() {
            let len: u32 = match main_function.cards.len().try_into() {
                Ok(i) => i,
                Err(_) => return Err(self.error(CompilationErrorPayload::TooManyCards(il))),
            };
            self.current_index = CardIndex::new(il, 0);
            self.scope_begin();
            self.process_function(main_function)?;
            self.current_index = CardIndex {
                function: il,
                card_index: FunctionCardIndex {
                    indices: smallvec::smallvec![len],
                },
            };
            self.scope_end();
            // insert explicit exit after the first function
            self.process_card(&Card::Abort)?;
        }

        for (il, function) in functions {
            self.current_index = CardIndex::function(il);
            let nodeid_handle = self.current_index.as_handle();
            let handle = u32::try_from(self.program.bytecode.len())
                .expect("bytecode length to fit into 32 bits");
            self.program
                .labels
                .0
                .insert(nodeid_handle, Label::new(handle))
                .unwrap();

            self.scope_begin();
            self.process_function(function)?;
            self.scope_end();
            self.push_instruction(Instruction::ScalarNil);
            self.emit_return()?;
        }

        Ok(())
    }

    /// begin nested compile sequence
    fn compile_begin(&mut self) {
        self.function_id += 1;
        self.locals.push(Default::default());
        self.upvalues.push(Default::default());
        self.scope_depth.push(0);
    }

    /// end nested compile sequence
    fn compile_end(&mut self) {
        self.function_id -= 1;
        self.locals.pop();
        self.upvalues.pop();
        self.scope_depth.pop();
    }

    fn scope_begin(&mut self) {
        *self.scope_depth.last_mut().unwrap() += 1;
    }

    fn scope_depth(&self) -> i32 {
        *self.scope_depth.last().unwrap()
    }

    fn scope_depth_mut(&mut self) -> &mut i32 {
        self.scope_depth.last_mut().unwrap()
    }

    fn scope_end(&mut self) {
        *self.scope_depth_mut() -= 1;
        let scope_depth = self.scope_depth();
        let locals = &mut self.locals[self.function_id];
        while locals
            .last()
            .map(|l| l.depth > scope_depth)
            .unwrap_or(false)
        {
            let var = locals.pop().unwrap();
            if var.captured {
                self.program.bytecode.push(Instruction::CloseUpvalue as u8);
            } else {
                self.program.bytecode.push(Instruction::Pop as u8);
            }
        }
    }

    /// add a local variable
    ///
    /// return its index
    fn add_local(&mut self, name: &'a str) -> CompilationResult<u32> {
        self.validate_var_name(name)?;
        self.add_local_unchecked(name)
    }

    fn add_upvalue(
        &mut self,
        index: u8,
        is_local: bool,
        function_id: usize,
    ) -> CompilationResult<usize> {
        let upvalues = &mut self.upvalues[function_id];
        for (i, val) in upvalues.iter().enumerate() {
            if val.index == index && val.is_local == is_local {
                // do not add duplicates
                return Ok(i);
            }
        }
        let i = upvalues.len();
        upvalues.push(Upvalue { is_local, index });
        Ok(i)
    }

    fn add_local_unchecked(&mut self, name: &'a str) -> CompilationResult<u32> {
        let depth = self.scope_depth();
        let locals = self.locals.last_mut().unwrap();
        let result = locals.len();
        locals
            .try_push(Local {
                name,
                depth,
                captured: false,
            })
            .map_err(|_| self.error(CompilationErrorPayload::TooManyLocals))?;
        Ok(result as u32)
    }

    fn process_function(
        &mut self,
        FunctionIr {
            arguments,
            cards,
            namespace,
            imports,
            ..
        }: &'a FunctionIr,
    ) -> CompilationResult<()> {
        self.current_namespace = Cow::Borrowed(namespace);
        self.current_imports = Cow::Borrowed(imports);

        // at runtime: pop arguments reverse order as the variables were declared
        for param in arguments.iter().rev() {
            self.add_local(param.as_str())?;
        }
        for (ic, card) in cards.iter().enumerate() {
            // valid indices always have 1 subindex, so replace that
            self.current_index.pop_subindex();
            self.current_index.push_subindex(ic as u32);
            self.process_card(card)?;
        }
        Ok(())
    }

    fn emit_return(&mut self) -> CompilationResult<()> {
        self.push_instruction(Instruction::Return);
        Ok(())
    }

    /// encodes the then block (card), and conditionally jumps over it using the `skip_instr`
    fn encode_if_then(
        &mut self,
        skip_instr: Instruction,
        then: impl FnOnce(&mut Self) -> CompilationResult<()>,
    ) -> CompilationResult<()> {
        type Pos = i32;
        debug_assert!(
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
            std::ptr::write_unaligned(ptr, self.program.bytecode.len() as Pos);
        };
        Ok(())
    }

    fn encode_jump(&mut self, function: &str) -> CompilationResult<()> {
        let to = self.lookup_function(&self.jump_table, function)?;
        write_to_vec(to.hash_key, &mut self.program.bytecode);
        write_to_vec(to.arity, &mut self.program.bytecode);
        Ok(())
    }

    // take jump_table by param because of lifetimes
    fn lookup_function<'b>(
        &self,
        jump_table: &'b CaoHashMap<String, FunctionMeta>,
        function: &str,
    ) -> CompilationResult<&'b FunctionMeta> {
        // attempt to look up the function by name
        let mut to = jump_table.get(function);
        if to.is_none() {
            // attempt to look up the function in the current namespace

            // current_namespace.join('.').push('.').push_str(function.0)
            let name = self
                .current_namespace
                .iter()
                .flat_map(|x| [x.as_ref(), "."])
                .chain(std::iter::once(function))
                .collect::<String>();

            to = jump_table.get(&name);
        }
        if to.is_none() {
            // attempt to look up function in the imports
            if let Some((_, alias)) = self
                .current_imports
                .iter()
                .find(|(import, _)| *import == function)
            {
                let (super_depth, suffix) = super_depth(alias);
                let name = self
                    .current_namespace
                    .iter()
                    .take(self.current_namespace.len() - super_depth)
                    .flat_map(|x| [x.as_ref(), "."])
                    .chain(std::iter::once(suffix.unwrap_or(alias)))
                    .collect::<String>();

                to = jump_table.get(&name);
            }
        }
        if to.is_none() {
            // attempt to look up by imported module
            if let Some((prefix, suffix)) = function.split_once('.') {
                if let Some((_, alias)) = self
                    .current_imports
                    .iter()
                    .find(|(import, _)| *import == prefix)
                {
                    // namespace.alias.suffix
                    let (super_depth, s) = super_depth(alias);

                    let name = self
                        .current_namespace
                        .iter()
                        .take(self.current_namespace.len() - super_depth)
                        .flat_map(|x| [x.as_ref(), "."])
                        .chain([alias, ".", s.unwrap_or(suffix)].iter().copied())
                        .collect::<String>();

                    to = jump_table.get(&name);
                }
            }
        }
        to.ok_or_else(|| {
            self.error(CompilationErrorPayload::InvalidJump {
                dst: function.to_string(),
                msg: None,
            })
        })
    }

    fn push_str(&mut self, data: &str) {
        let handle = self.program.data.len() as u32;
        write_to_vec(handle, &mut self.program.bytecode);
        encode_str(data, &mut self.program.data);
    }

    /// function_id = index in the *offset array
    fn resolve_upvalue(&mut self, name: &str, function_id: usize) -> CompilationResult<Variable> {
        if function_id == 0 {
            return Ok(Variable::Global);
        }

        let locals = &mut self.locals[function_id - 1];
        // try to find in the locals of the parent function
        for (i, local) in locals.iter_mut().enumerate() {
            if local.name == name {
                local.captured = true;
                return self
                    .add_upvalue(i as u8, true, function_id)
                    .map(Variable::Upvalue);
            }
        }
        if function_id == 0 {
            Ok(Variable::Global)
        } else {
            let val = self.resolve_upvalue(name, function_id - 1)?;
            if let Variable::Upvalue(i) = &val {
                // if the variable is an upvalue in the enclosing function add a non-local upvalue to
                // this function
                let i = self.add_upvalue(*i as u8, false, function_id)?;
                return Ok(Variable::Upvalue(i));
            }
            Ok(val)
        }
    }

    fn resolve_var(&mut self, name: &str) -> CompilationResult<Variable> {
        self.validate_var_name(name)?;

        for (i, local) in self.locals[self.function_id].iter_mut().enumerate().rev() {
            if local.name == name {
                return Ok(Variable::Local(i));
            }
        }
        self.resolve_upvalue(name, self.function_id)
    }

    fn process_card(&mut self, card: &'a Card) -> CompilationResult<()> {
        let card_byte_index = u32::try_from(self.program.bytecode.len())
            .expect("Expected bytecode length to fit into 32 bits");
        let nodeid_hash = self.current_index.as_handle();
        self.program
            .labels
            .0
            .insert(nodeid_hash, Label::new(card_byte_index))
            .unwrap();

        match card {
            Card::CompositeCard(comp) => {
                for (i, card) in comp.cards.iter().enumerate() {
                    self.current_index.push_subindex(i as u32);
                    self.process_card(card)?;
                    self.current_index.pop_subindex()
                }
            }
            Card::ForEach(fe) => {
                let ForEach {
                    i,
                    k,
                    v,
                    iterable: variable,
                    body,
                } = fe.as_ref();
                self.current_index.push_subindex(0);
                self.process_card(variable)?;
                self.current_index.pop_subindex();

                self.scope_begin();
                let loop_var = self.add_local_unchecked("")?;
                let loop_item = self.add_local_unchecked("")?;
                // ForEach instruction will push these values on the stack
                let v = match v {
                    Some(o) => self.add_local(&o)?,
                    None => self.add_local_unchecked("")?,
                };
                let k = match k {
                    Some(k) => self.add_local(&k)?,
                    None => self.add_local_unchecked("")?,
                };
                let i = match i {
                    Some(i) => self.add_local(&i)?,
                    None => self.add_local_unchecked("")?,
                };
                self.push_instruction(Instruction::BeginForEach);
                write_to_vec(loop_var, &mut self.program.bytecode);
                write_to_vec(loop_item, &mut self.program.bytecode);

                let block_begin = self.program.bytecode.len() as i32;
                self.push_instruction(Instruction::ForEach);
                write_to_vec(loop_var, &mut self.program.bytecode);
                write_to_vec(loop_item, &mut self.program.bytecode);
                write_to_vec(i, &mut self.program.bytecode);
                write_to_vec(k, &mut self.program.bytecode);
                write_to_vec(v, &mut self.program.bytecode);
                self.encode_if_then(Instruction::GotoIfFalse, |c| {
                    c.current_index.push_subindex(1);
                    c.process_card(body)?;
                    c.current_index.pop_subindex();
                    // return to the foreach instruction
                    c.push_instruction(Instruction::Goto);
                    write_to_vec(block_begin, &mut c.program.bytecode);
                    Ok(())
                })?;
                self.scope_end();
            }
            Card::While(children) => {
                let [condition, body] = &**children;
                let block_begin = self.program.bytecode.len() as i32;
                self.current_index.push_subindex(0);
                self.process_card(condition)?;
                self.current_index.pop_subindex();
                self.current_index.push_subindex(1);
                // if false jump over the body block
                self.encode_if_then(Instruction::GotoIfFalse, |c| {
                    // if true execute body and jump to block_begin
                    c.process_card(body)?;
                    c.push_instruction(Instruction::Goto);
                    write_to_vec(block_begin, &mut c.program.bytecode);
                    Ok(())
                })?;
                self.current_index.pop_subindex();
            }
            Card::Repeat(rep) => {
                self.compile_subexpr(slice::from_ref(&rep.n))?;
                let i = &rep.i;
                let repeat = &rep.body;
                self.scope_begin();
                let loop_n_index = self.add_local_unchecked("")?;
                let loop_counter_index = self.add_local_unchecked("")?;
                let i_index = match i {
                    Some(var) => {
                        let index = self.add_local(&var)?;
                        Some(index)
                    }
                    None => None,
                };
                self.write_local_var(loop_n_index);
                // init counter to 0
                self.process_card(&Card::ScalarInt(0))?;
                self.write_local_var(loop_counter_index);

                let block_begin = self.program.bytecode.len() as i32;
                // loop condition
                self.read_local_var(loop_counter_index);
                self.read_local_var(loop_n_index);
                self.push_instruction(Instruction::Less);
                // loop body
                self.encode_if_then(Instruction::GotoIfFalse, |c| {
                    if let Some(i_index) = i_index {
                        c.read_local_var(loop_counter_index);
                        c.write_local_var(i_index);
                    }
                    c.current_index.push_subindex(0);
                    c.process_card(repeat)?;
                    c.current_index.pop_subindex();
                    // i = i + 1
                    c.process_card(&Card::ScalarInt(1))?;
                    c.read_local_var(loop_counter_index);
                    c.push_instruction(Instruction::Add);
                    c.write_local_var(loop_counter_index);
                    // return to the repeat instruction
                    c.push_instruction(Instruction::Goto);
                    write_to_vec(block_begin, &mut c.program.bytecode);
                    Ok(())
                })?;
                self.scope_end();
            }
            Card::ReadVar(variable) => {
                self.read_var_card(variable)?;
            }
            Card::SetVar(var) => {
                self.compile_subexpr(slice::from_ref(&var.value))?;
                let var = var.name.as_str();
                match var.rsplit_once('.') {
                    Some((read_props, set_prop)) => {
                        self.read_var_card(read_props)?;
                        self.push_instruction(Instruction::StringLiteral);
                        self.push_str(set_prop);
                        self.push_instruction(Instruction::SetProperty);
                    }
                    None => {
                        let index = match self.resolve_var(var)? {
                            Variable::Local(i) => i as u32,
                            Variable::Global => self.add_local(&var)?,
                            Variable::Upvalue(i) => {
                                self.write_upvalue(i as u32);
                                return Ok(());
                            }
                        };

                        self.write_local_var(index);
                    }
                }
            }
            Card::SetGlobalVar(var) => {
                self.compile_subexpr(slice::from_ref(&var.value))?;
                self.push_instruction(Instruction::SetGlobalVar);
                let variable = var.name.as_str();
                let next_var = &mut self.next_var;
                if variable.is_empty() {
                    return Err(self.error(CompilationErrorPayload::EmptyVariable));
                }
                let varhash = Handle::from_bytes(variable.as_bytes());

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
                    .or_insert_with(move || variable.to_string());
                write_to_vec(*id, &mut self.program.bytecode);
            }
            Card::IfElse(children) => {
                let [condition, then_card, else_card] = &**children;
                self.compile_subexpr(slice::from_ref(condition))?;

                let mut idx = 0;
                self.current_index.push_subindex(1);
                self.encode_if_then(Instruction::GotoIfFalse, |c| {
                    c.process_card(then_card)?;
                    // jump over the `else` branch
                    c.push_instruction(Instruction::Goto);
                    idx = c.program.bytecode.len();
                    write_to_vec(0xEEFi32, &mut c.program.bytecode);
                    Ok(())
                })?;
                self.current_index.pop_subindex();
                self.current_index.push_subindex(2);
                self.process_card(else_card)?;
                self.current_index.pop_subindex();
                unsafe {
                    let ptr = self.program.bytecode.as_mut_ptr().add(idx) as *mut i32;
                    std::ptr::write_unaligned(ptr, self.program.bytecode.len() as i32);
                }
            }
            Card::IfFalse(jmp) => {
                let [cond, body] = &**jmp;
                self.compile_subexpr(slice::from_ref(cond))?;
                self.current_index.push_subindex(1);
                self.encode_if_then(Instruction::GotoIfTrue, |c| c.process_card(body))?;
                self.current_index.pop_subindex();
            }
            Card::IfTrue(jmp) => {
                let [cond, body] = &**jmp;
                self.compile_subexpr(slice::from_ref(cond))?;
                self.current_index.push_subindex(1);
                self.encode_if_then(Instruction::GotoIfFalse, |c| c.process_card(body))?;
                self.current_index.pop_subindex();
            }
            Card::Call(jmp) => {
                self.compile_subexpr(&jmp.args.0)?;
                self.push_instruction(Instruction::FunctionPointer);
                self.encode_jump(jmp.function_name.as_str())?;
                self.push_instruction(Instruction::CallFunction);
            }
            Card::StringLiteral(c) => {
                self.push_instruction(Instruction::StringLiteral);
                self.push_str(c.as_str())
            }
            Card::CallNative(c) => {
                self.compile_subexpr(&c.args.0)?;
                let name = &c.name;
                let key = Handle::from_str(name.as_str()).unwrap();
                self.push_instruction(Instruction::CallNative);
                write_to_vec(key, &mut self.program.bytecode);
            }
            Card::ScalarInt(s) => {
                self.push_instruction(Instruction::ScalarInt);
                write_to_vec(*s, &mut self.program.bytecode);
            }
            Card::ScalarFloat(s) => {
                self.push_instruction(Instruction::ScalarFloat);
                write_to_vec(*s, &mut self.program.bytecode);
            }
            Card::Function(fname) => {
                self.push_instruction(Instruction::FunctionPointer);
                self.encode_jump(fname)?;
            }
            Card::Closure(embedded_function) => {
                // jump over the inner function
                // yes, this is cursed
                // no, I don't care anymore
                self.push_instruction(Instruction::Goto);
                let goto_index = self.program.bytecode.len();
                write_to_vec(0xEEFi32, &mut self.program.bytecode);

                self.compile_begin();
                let function_handle = self.current_index.as_handle();
                let arity = embedded_function.arguments.len() as u32;
                let handle = u32::try_from(self.program.bytecode.len())
                    .expect("bytecode length to fit into 32 bits");
                self.program
                    .labels
                    .0
                    .insert(function_handle, Label::new(handle))
                    .unwrap();

                // process the embedded function inline
                self.scope_begin();
                // TODO: dedupe these loops w/ process_function?
                // at runtime: pop arguments reverse order as the variables were declared
                for param in embedded_function.arguments.iter().rev() {
                    self.add_local(param.as_str())?;
                }
                self.compile_subexpr(&embedded_function.cards)?;
                self.scope_end();
                self.push_instruction(Instruction::ScalarNil);
                self.emit_return()?;

                // finish the goto that jumps over the inner function
                unsafe {
                    let ptr = self.program.bytecode.as_mut_ptr().add(goto_index) as *mut i32;
                    std::ptr::write_unaligned(ptr, self.program.bytecode.len() as i32);
                }

                // finally, push the closure instruction
                // the goto instruction will jump here
                self.push_instruction(Instruction::Closure);
                write_to_vec(function_handle, &mut self.program.bytecode);
                write_to_vec(arity, &mut self.program.bytecode);
                let upvalues = std::mem::take(&mut self.upvalues[self.function_id]);
                for upvalue in upvalues {
                    self.push_instruction(Instruction::CopyLast);
                    self.push_instruction(Instruction::RegisterUpvalue);
                    write_to_vec(upvalue.index, &mut self.program.bytecode);
                    write_to_vec(upvalue.is_local as u8, &mut self.program.bytecode);
                }
                self.compile_end();
            }
            Card::NativeFunction(fname) => {
                self.push_instruction(Instruction::NativeFunctionPointer);
                self.push_str(fname.as_str());
            }
            Card::Array(expressions) => {
                // create a table, then for each sub-card: insert the subcard and append it to the
                // result
                // finally: ensure the result is on the stack
                self.push_instruction(Instruction::InitTable);
                let table_var = self.add_local_unchecked("")?;
                self.write_local_var(table_var);
                for (i, card) in expressions.iter().enumerate() {
                    // push nil, so if the card results in no output,
                    // we append nil to the table
                    self.push_instruction(Instruction::ScalarNil);
                    self.current_index.push_subindex(i as u32);
                    self.process_card(card)?;
                    self.current_index.pop_subindex();
                    self.read_local_var(table_var);
                    self.push_instruction(Instruction::AppendTable);
                }
                // push the table to the stack
                self.read_local_var(table_var);
            }
            Card::Len(expr) => {
                self.compile_subexpr(slice::from_ref(expr.card.as_ref()))?;
                self.push_instruction(Instruction::Len);
            }
            Card::Return(expr) => {
                self.compile_subexpr(slice::from_ref(expr.card.as_ref()))?;
                self.emit_return()?;
            }
            Card::Not(expr) => {
                self.compile_subexpr(slice::from_ref(expr.card.as_ref()))?;
                self.push_instruction(Instruction::Not);
            }
            Card::Get(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::NthRow);
            }
            Card::And(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::And);
            }
            Card::Or(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Or);
            }
            Card::Xor(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Xor);
            }
            Card::Equals(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Equals);
            }
            Card::Less(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Less);
            }
            Card::LessOrEq(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::LessOrEq);
            }
            Card::NotEquals(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::NotEquals);
            }
            Card::Add(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Add);
            }
            Card::Sub(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Sub);
            }
            Card::Mul(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Mul);
            }
            Card::Div(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::Div);
            }
            Card::GetProperty(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::GetProperty);
            }
            Card::SetProperty(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::SetProperty);
            }
            Card::AppendTable(expr) => {
                self.compile_subexpr(expr.as_ref())?;
                self.push_instruction(Instruction::AppendTable);
            }
            Card::PopTable(expr) => {
                self.compile_subexpr(slice::from_ref(expr.card.as_ref()))?;
                self.push_instruction(Instruction::PopTable);
            }
            Card::DynamicCall(jump) => {
                self.compile_subexpr(jump.args.0.as_slice())?;
                self.current_index.push_subindex(jump.args.0.len() as u32);
                self.process_card(&jump.function)?;
                self.current_index.pop_subindex();
                self.push_instruction(Instruction::CallFunction);
            }
            Card::ScalarNil => {
                self.push_instruction(Instruction::ScalarNil);
            }
            Card::Abort => {
                self.push_instruction(Instruction::Exit);
            }
            Card::CreateTable => {
                self.push_instruction(Instruction::InitTable);
            }
        }
        Ok(())
    }

    fn read_var_card(&mut self, mut variable: &str) -> CompilationResult<()> {
        // sub properties on a table
        let props = match variable.split_once('.') {
            Some((v, props)) => {
                variable = v;
                props
            }
            None => "",
        };
        let scope = self.resolve_var(variable)?;
        match scope {
            Variable::Local(index) => {
                self.read_local_var(index as u32);
            }
            Variable::Upvalue(index) => {
                self.read_upvalue(index as u32);
            }
            Variable::Global => {
                let next_var = &mut self.next_var;
                let varhash = Handle::from_bytes(variable.as_bytes());
                let id = *self
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
                    .or_insert_with(|| variable.to_string());
                self.push_instruction(Instruction::ReadGlobalVar);
                write_to_vec(id, &mut self.program.bytecode);
            }
        }
        // handle props
        for prop in props.split(".").filter(|p| !p.is_empty()) {
            self.push_instruction(Instruction::StringLiteral);
            self.push_str(prop);
            self.push_instruction(Instruction::GetProperty);
        }
        Ok(())
    }

    fn read_local_var(&mut self, index: u32) {
        self.push_instruction(Instruction::ReadLocalVar);
        write_to_vec(index, &mut self.program.bytecode);
    }

    fn write_local_var(&mut self, index: u32) {
        self.push_instruction(Instruction::SetLocalVar);
        write_to_vec(index, &mut self.program.bytecode);
    }

    fn read_upvalue(&mut self, index: u32) {
        self.push_instruction(Instruction::ReadUpvalue);
        write_to_vec(index, &mut self.program.bytecode);
    }

    fn write_upvalue(&mut self, index: u32) {
        self.push_instruction(Instruction::SetUpvalue);
        write_to_vec(index, &mut self.program.bytecode);
    }

    fn validate_var_name(&self, name: &str) -> CompilationResult<()> {
        if name.is_empty() {
            return Err(self.error(CompilationErrorPayload::EmptyVariable));
        }
        Ok(())
    }

    fn push_instruction(&mut self, instruction: Instruction) {
        self.program
            .trace
            .insert(self.program.bytecode.len() as u32, self.trace())
            .unwrap();
        self.program.bytecode.push(instruction as u8);
    }

    fn compile_subexpr(&mut self, cards: &'a [Card]) -> CompilationResult<()> {
        for (i, card) in cards.iter().enumerate() {
            self.current_index.push_subindex(i as u32);
            self.process_card(card)?;
            self.current_index.pop_subindex();
        }
        Ok(())
    }
}

fn super_depth(import: &str) -> (usize, Option<&str>) {
    let mut super_pog = import.split_once("super.");
    let mut super_cnt = 0;
    let mut suffix = None;
    while let Some((_sup_pre, sup_post)) = super_pog {
        super_cnt += 1;
        super_pog = sup_post.split_once("super.");
        suffix = Some(sup_post);
    }

    (super_cnt, suffix)
}
