pub mod data;
pub mod instr_execution;

#[cfg(test)]
mod tests;

use crate::{binary_compare, pop_stack};
use crate::{collections::pre_hash_map::Key, instruction::Instruction};
use crate::{collections::pre_hash_map::PreHashMap, prelude::*};
use crate::{collections::stack::ScalarStack, VariableId};
use crate::{scalar::Scalar, InputString};
use data::RuntimeData;
use serde::{Deserialize, Serialize};
use slog::{debug, trace, warn};
use slog::{o, Drain, Logger};
use std::collections::HashMap;
use std::mem::transmute;

type ConvertFn<Aux> = unsafe fn(&Object, &VM<Aux>) -> Box<dyn ObjectProperties>;

#[derive(Debug, Clone, Copy)]
pub enum ConvertError {
    /// Null object was passed to convert
    NullPtr,
    BadType,
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    /// nullable index of the Object's data in the VM memory
    pub index: Option<Pointer>,
    /// size of the data in the VM memory
    pub size: u32,
}

impl Default for Object {
    fn default() -> Self {
        Self::null()
    }
}

impl Object {
    pub fn null() -> Self {
        Self {
            index: None,
            size: 0,
        }
    }

    pub fn as_inner<Aux>(&self, vm: &VM<Aux>) -> Result<Box<dyn ObjectProperties>, ConvertError> {
        self.index
            .ok_or(ConvertError::NullPtr)
            .map(|index| unsafe { vm.converters[&index](self, vm) })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub struct HistoryEntry {
    pub id: NodeId,
    pub instr: Option<Instruction>,
}

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary data structure passed to custom functions.
pub struct VM<'a, Aux = ()>
where
    Aux: 'a,
{
    pub logger: Logger,
    pub history: Vec<HistoryEntry>,
    pub auxiliary_data: Aux,
    pub max_iter: i32,

    pub runtime_data: RuntimeData,

    callables: PreHashMap<Procedure<Aux>>,
    objects: HashMap<Pointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<Pointer, ConvertFn<Aux>>,
    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> VM<'a, Aux> {
    pub fn new(logger: impl Into<Option<Logger>>, auxiliary_data: Aux) -> Self {
        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        Self {
            logger,
            history: Vec::new(),
            converters: HashMap::new(),
            auxiliary_data,
            callables: PreHashMap::default(),
            objects: HashMap::with_capacity(128),
            runtime_data: RuntimeData {
                memory_limit: 40000,
                memory: Vec::with_capacity(512),
                stack: ScalarStack::new(256),
                registers: Vec::with_capacity(128),
            },
            max_iter: 1000,
            _m: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.converters.clear();

        self.runtime_data.clear();
    }

    pub fn read_var(&self, name: VariableId) -> Option<&Scalar> {
        self.runtime_data.registers.get(name.0 as usize)
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_iter = max_iter;
        self
    }

    pub fn stack(&self) -> &[Scalar] {
        self.runtime_data.stack.as_slice()
    }

    pub fn get_aux(&self) -> &Aux {
        &self.auxiliary_data
    }

    pub fn get_aux_mut(&mut self) -> &mut Aux {
        &mut self.auxiliary_data
    }

    pub fn unwrap_aux(self) -> Aux {
        self.auxiliary_data
    }

    pub fn register_function<C: Callable<Aux> + 'static>(&mut self, name: InputString, f: C) {
        let hash = Key::from_str(name.as_str());
        self.callables.insert(hash, Procedure::new(name, f));
    }

    pub fn register_function_obj(&mut self, name: &str, f: Procedure<Aux>) {
        let hash = Key::from_str(name);
        self.callables.insert(hash, f);
    }

    pub fn get_value_in_place<T: DecodeInPlace<'a>>(
        &'a self,
        bytecode_pos: Pointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        let object = self.objects.get(&bytecode_pos)?;

        match self.runtime_data.get_value_in_place::<T>(object) {
            Some(val) => Some(val),
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, bytecode_pos: Pointer) -> Option<T> {
        let object = self.objects.get(&bytecode_pos)?;
        match object.index {
            Some(index) => {
                let data = &self.runtime_data.memory;
                let head = index.0 as usize;
                let tail = (head.checked_add(object.size as usize))
                    .unwrap_or(data.len())
                    .min(data.len());
                T::decode(&data[head..tail]).ok().map(|(_, val)| val)
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value_with_decoder<T: ByteEncodeProperties>(
        &mut self,
        val: T,
        converter: ConvertFn<Aux>,
    ) -> Result<Object, ExecutionError> {
        let (index, size) = self.runtime_data.write_to_memory(val)?;
        let object = Object {
            index: Some(index),
            size: size as u32,
        };
        self.objects.insert(index, object);
        self.converters.insert(index as Pointer, converter);

        self.stack_push(Scalar::Pointer(index as Pointer))?;

        debug!(self.logger, "Set value {:?} {}", object, T::displayname());

        Ok(object)
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value<T: ByteEncodeProperties + ByteDecodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<Object, ExecutionError> {
        let (index, size) = self.runtime_data.write_to_memory(val)?;
        let object = Object {
            index: Some(index),
            size: size as u32,
        };
        self.objects.insert(index, object);
        self.converters
            .insert(index as Pointer, |o: &Object, vm: &VM<Aux>| {
                let res: T = vm.get_value(o.index.unwrap()).unwrap();
                Box::new(res)
            });

        self.stack_push(Scalar::Pointer(index as Pointer))?;

        debug!(self.logger, "Set value {:?} {}", object, T::displayname());

        Ok(object)
    }

    pub fn stack_push<S>(&mut self, value: S) -> Result<(), ExecutionError>
    where
        S: Into<Scalar>,
    {
        self.runtime_data
            .stack
            .push(value.into())
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Option<Scalar> {
        Some(self.runtime_data.stack.pop())
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is fairly unsafe
    pub fn run(&mut self, program: &'a CompiledProgram) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Running program");
        self.history.clear();
        let mut bytecode_pos = 0;
        let len = program.bytecode.len();
        let mut max_iter = self.max_iter;
        while bytecode_pos < len {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = unsafe { *program.bytecode.as_ptr().offset(bytecode_pos as isize) };
            let instr = unsafe { transmute(instr) };
            trace!(
                self.logger,
                "Instruction: {:?}({:?}) Pointer: {:?}",
                instr,
                program.bytecode[bytecode_pos],
                bytecode_pos
            );
            bytecode_pos += 1;
            match instr {
                Instruction::Breadcrumb => instr_execution::instr_breadcrumb(
                    &self.logger,
                    &mut self.history,
                    &program.bytecode,
                    &mut bytecode_pos,
                ),
                Instruction::ClearStack => {
                    self.runtime_data.stack.clear();
                }
                Instruction::SetVar => {
                    instr_execution::instr_set_var(
                        &self.logger,
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut bytecode_pos,
                    )?;
                }
                Instruction::ReadVar => {
                    instr_execution::instr_read_var(
                        &self.logger,
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut bytecode_pos,
                    )?;
                }
                Instruction::Pop => {
                    if self.runtime_data.stack.len() > 0 {
                        self.runtime_data.stack.pop();
                    }
                }
                Instruction::Jump => {
                    instr_execution::instr_jump(&self.logger, &mut bytecode_pos, program)?;
                }
                Instruction::Exit => {
                    return instr_execution::instr_exit(&self.logger, &mut self.runtime_data)
                }
                Instruction::JumpIfTrue => {
                    instr_execution::jump_if(
                        &self.logger,
                        &mut self.runtime_data,
                        &mut bytecode_pos,
                        program,
                        |s| s.as_bool(),
                    )?;
                }
                Instruction::JumpIfFalse => {
                    instr_execution::jump_if(
                        &self.logger,
                        &mut self.runtime_data,
                        &mut bytecode_pos,
                        program,
                        |s| !s.as_bool(),
                    )?;
                }
                Instruction::CopyLast => {
                    let val = self.runtime_data.stack.last();
                    self.runtime_data
                        .stack
                        .push(val)
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::Pass => {}
                Instruction::ScalarLabel => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            instr_execution::decode_value(
                                &self.logger,
                                &program.bytecode,
                                &mut bytecode_pos,
                            )
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            instr_execution::decode_value(
                                &self.logger,
                                &program.bytecode,
                                &mut bytecode_pos,
                            )
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Floating(unsafe {
                            instr_execution::decode_value(
                                &self.logger,
                                &program.bytecode,
                                &mut bytecode_pos,
                            )
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarArray => instr_execution::instr_scalar_array(
                    &self.logger,
                    &mut self.runtime_data,
                    &program.bytecode,
                    &mut bytecode_pos,
                )?,
                Instruction::Add => self.binary_op(|a, b| a + b)?,
                Instruction::Sub => self.binary_op(|a, b| a - b)?,
                Instruction::Mul => self.binary_op(|a, b| a * b)?,
                Instruction::Div => self.binary_op(|a, b| a / b)?,
                Instruction::Equals => binary_compare!(self, ==, false),
                Instruction::NotEquals => binary_compare!(self, !=, true),
                Instruction::Less => binary_compare!(self, <, false),
                Instruction::LessOrEq => binary_compare!(self, <=, false),
                Instruction::StringLiteral => instr_execution::instr_string_literal(
                    self,
                    &mut bytecode_pos,
                    &program.bytecode,
                )?,
                Instruction::Call => {
                    instr_execution::execute_call(self, &mut bytecode_pos, &program.bytecode)?
                }
            }
            debug!(
                self.logger,
                "Stack len: {} {:?} ptr: {}",
                self.runtime_data.stack.len(),
                self.log_stack(),
                bytecode_pos
            );
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    pub fn log_stack(&self) {
        trace!(self.logger, "--------Stack--------");
        for s in self.runtime_data.stack.as_slice().iter().rev() {
            trace!(self.logger, "{:?}", s);
        }
        trace!(self.logger, "------End Stack------");
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ExecutionError>
    where
        F: Fn(Scalar, Scalar) -> Scalar,
    {
        let b = pop_stack!(self);
        let a = pop_stack!(self);

        self.runtime_data
            .stack
            .push(op(a, b))
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }
}
