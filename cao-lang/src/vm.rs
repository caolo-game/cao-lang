//! Cao-Lang back-end
//!
//! Interprets the compiled output produced by the Cao-Lang compiler
pub mod data;
mod instr_execution;

#[cfg(test)]
mod tests;

use crate::{binary_compare, pop_stack};
use crate::{
    collections::bounded_stack::BoundedStack, collections::scalar_stack::ScalarStack, VariableId,
};
use crate::{collections::pre_hash_map::Key, instruction::Instruction};
use crate::{collections::pre_hash_map::PreHashMap, prelude::*};
use crate::{scalar::Scalar, InputString};
use data::RuntimeData;
use std::{collections::HashMap, str::FromStr};
use std::{convert::TryFrom, mem::transmute};

type ConvertFn<Aux> = unsafe fn(&Object, &Vm<Aux>) -> Box<dyn ObjectProperties>;

#[derive(Debug, Clone, Copy)]
pub enum ConvertError {
    /// Null object was passed to convert
    NullPtr,
    BadType,
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    /// nullable index of the Object's data in the Vm memory
    pub index: Option<Pointer>,
    /// size of the data in the Vm memory
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

    pub fn as_inner<Aux>(&self, vm: &Vm<Aux>) -> Result<Box<dyn ObjectProperties>, ConvertError> {
        self.index
            .ok_or(ConvertError::NullPtr)
            .map(|index| unsafe { vm.converters[&index](self, vm) })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoryEntry {
    pub id: NodeId,
    pub instr: Option<Instruction>,
}

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary data structure passed to custom functions.
pub struct Vm<'a, Aux = ()>
where
    Aux: 'a,
{
    /// Breadcrumb instructions will populat this history log.
    pub history: Vec<HistoryEntry>,
    pub auxiliary_data: Aux,
    /// Number of instructions `run` will execute before returning Timeout
    pub max_iter: i32,

    pub runtime_data: RuntimeData,

    callables: PreHashMap<Procedure<Aux>>,
    objects: HashMap<Pointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<Pointer, ConvertFn<Aux>>,

    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> Vm<'a, Aux> {
    pub fn new(auxiliary_data: Aux) -> Self {
        Self {
            history: Vec::new(),
            converters: HashMap::new(),
            auxiliary_data,
            callables: PreHashMap::default(),
            objects: HashMap::with_capacity(128),
            runtime_data: RuntimeData {
                memory_limit: 40000,
                memory: Vec::with_capacity(512),
                stack: ScalarStack::new(256),
                global_vars: Vec::with_capacity(128),
                return_stack: BoundedStack::new(256),
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

    #[inline]
    pub fn read_var(&self, name: VariableId) -> Option<Scalar> {
        self.runtime_data.global_vars.get(name.0 as usize).cloned()
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_iter = max_iter;
        self
    }

    #[inline]
    pub fn get_aux(&self) -> &Aux {
        &self.auxiliary_data
    }

    #[inline]
    pub fn get_aux_mut(&mut self) -> &mut Aux {
        &mut self.auxiliary_data
    }

    #[inline]
    pub fn unwrap_aux(self) -> Aux {
        self.auxiliary_data
    }

    pub fn register_function<C: Callable<Aux> + 'static>(&mut self, name: InputString, f: C) {
        let key = Key::from_str(name.as_str()).unwrap();
        self.callables.insert(key, Procedure::new(name, f));
    }

    pub fn get_value_in_place<T: DecodeInPlace<'a>>(
        &'a self,
        bytecode_pos: Pointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        let object = self.objects.get(&bytecode_pos)?;

        self.runtime_data.get_value_in_place::<T>(object)
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, bytecode_pos: Pointer) -> Option<T> {
        let object = self.objects.get(&bytecode_pos)?;
        object.index.and_then(|index| {
            let data = &self.runtime_data.memory;
            let head = index.0 as usize;
            let tail = (head.checked_add(object.size as usize))
                .unwrap_or(data.len())
                .min(data.len());
            T::decode(&data[head..tail]).ok().map(|(_, val)| val)
        })
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
            .insert(index as Pointer, |o: &Object, vm: &Vm<Aux>| {
                let res: T = vm.get_value(o.index.unwrap()).unwrap();
                Box::new(res)
            });

        self.stack_push(Scalar::Pointer(index as Pointer))?;

        Ok(object)
    }

    #[inline]
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

    #[inline]
    pub fn stack_pop(&mut self) -> Scalar {
        self.runtime_data.stack.pop()
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is fairly unsafe
    pub fn run(&mut self, program: &CompiledProgram) -> Result<i32, ExecutionError> {
        self.history.clear();
        let mut bytecode_pos = 0;
        let len = program.bytecode.len();
        let mut remaining_iters = self.max_iter;
        while bytecode_pos < len {
            remaining_iters -= 1;
            if remaining_iters <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr: u8 = unsafe { *program.bytecode.as_ptr().add(bytecode_pos) };
            let instr: Instruction = unsafe { transmute(instr) };
            bytecode_pos += 1;
            match instr {
                Instruction::GotoIfTrue => {
                    let condition = self.runtime_data.stack.pop();
                    let pos = self.runtime_data.stack.pop();
                    let pos: i32 = match i32::try_from(pos) {
                        Ok(i) => i,
                        Err(err) => {
                            return Err(ExecutionError::InvalidArgument {
                                context: Some(format!(
                                    "Goto instruction got invalid position value {:?}",
                                    err
                                )),
                            })
                        }
                    };
                    if condition.as_bool() {
                        bytecode_pos = pos as usize;
                    }
                }
                Instruction::Goto => {
                    let pos = self.runtime_data.stack.pop();
                    let pos: i32 = match i32::try_from(pos) {
                        Ok(i) => i,
                        Err(err) => {
                            return Err(ExecutionError::InvalidArgument {
                                context: Some(format!(
                                    "Goto instruction got invalid position value {:?}",
                                    err
                                )),
                            })
                        }
                    };
                    bytecode_pos = pos as usize;
                }
                Instruction::SwapLast => {
                    let b = pop_stack!(self);
                    let a = pop_stack!(self);
                    // we popped two values, we know that the stack has capacity for 2 ..
                    self.stack_push(b).unwrap();
                    self.stack_push(a).unwrap();
                }
                Instruction::Remember => {
                    //
                    //
                    // TODO: we could use the Sentinel values to store the return addresses instead
                    // of a separate call stack ?
                    //
                    //
                    //
                    let offset = self.runtime_data.stack.pop();
                    let offset: i32 = match i32::try_from(offset) {
                        Ok(i) => i,
                        Err(err) => {
                            return Err(ExecutionError::InvalidArgument {
                                context: Some(format!(
                                    "Remember instruction got invalid offset value {:?}",
                                    err
                                )),
                            })
                        }
                    };

                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(
                            (bytecode_pos as isize + offset as isize) as i32,
                        ))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarNull => self.stack_push(Scalar::Null)?,
                Instruction::Breadcrumb => instr_execution::instr_breadcrumb(
                    &mut self.history,
                    &program.bytecode,
                    &mut bytecode_pos,
                ),
                Instruction::ClearStack => {
                    self.runtime_data.stack.clear_until_sentinel();
                }
                Instruction::SetGlobalVar => {
                    instr_execution::instr_set_var(
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut bytecode_pos,
                    )?;
                }
                Instruction::ReadGlobalVar => {
                    instr_execution::instr_read_var(
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut bytecode_pos,
                    )?;
                }
                Instruction::Pop => {
                    if !self.runtime_data.stack.is_empty() {
                        self.runtime_data.stack.pop();
                    }
                }
                Instruction::Jump => {
                    instr_execution::instr_jump(
                        &mut bytecode_pos,
                        program,
                        &mut self.runtime_data,
                    )?;
                }
                Instruction::ScopeStart => {
                    self.runtime_data
                        .stack
                        .push_sentinel()
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScopeEnd => {
                    let scope_result = self.runtime_data.stack.clear_until_sentinel();
                    self.stack_push(scope_result).unwrap(); // we just popped this value, there must be capacity for storing it
                }
                Instruction::Return => match self.runtime_data.return_stack.pop() {
                    Some(ptr) => {
                        bytecode_pos = ptr;
                    }
                    None => {
                        return Err(ExecutionError::BadReturn {
                            reason: "Call stack is empty".to_string(),
                        });
                    }
                },
                Instruction::Exit => return instr_execution::instr_exit(&mut self.runtime_data),
                Instruction::JumpIfTrue => {
                    instr_execution::jump_if(
                        &mut self.runtime_data,
                        &mut bytecode_pos,
                        program,
                        |s| s.as_bool(),
                    )?;
                }
                Instruction::JumpIfFalse => {
                    instr_execution::jump_if(
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
                            instr_execution::decode_value(&program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Floating(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarArray => instr_execution::instr_scalar_array(
                    &mut self.runtime_data,
                    &program.bytecode,
                    &mut bytecode_pos,
                )?,
                Instruction::And => {
                    self.binary_op(|a, b| Scalar::from(a.as_bool() && b.as_bool()))?
                }
                Instruction::Or => {
                    self.binary_op(|a, b| Scalar::from(a.as_bool() || b.as_bool()))?
                }
                Instruction::Xor => {
                    self.binary_op(|a, b| Scalar::from(a.as_bool() ^ b.as_bool()))?
                }
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
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    #[inline]
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
