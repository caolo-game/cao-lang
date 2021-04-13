//! Cao-Lang back-end
//!
//! Interprets the compiled output produced by the Cao-Lang compiler
pub mod data;
mod instr_execution;

#[cfg(test)]
mod tests;

use crate::scalar::Scalar;
use crate::{binary_compare, pop_stack};
use crate::{
    collections::bounded_stack::BoundedStack, collections::scalar_stack::ScalarStack, VariableId,
};
use crate::{
    collections::pre_hash_map::{Key, PreHashMap},
    instruction::Instruction,
    prelude::*,
};
use data::RuntimeData;
use std::mem::transmute;
use std::str::FromStr;

use self::data::CallFrame;

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
            .map(|index| unsafe { vm.converters[index.0](self, vm) })
    }
}

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary data structure passed to custom functions.
pub struct Vm<'a, Aux = ()>
where
    Aux: 'a,
{
    pub auxiliary_data: Aux,
    /// Number of instructions `run` will execute before returning Timeout
    pub max_instr: i32,

    pub runtime_data: RuntimeData,

    callables: PreHashMap<Procedure<Aux>>,
    objects: PreHashMap<Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: PreHashMap<ConvertFn<Aux>>,

    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> Vm<'a, Aux> {
    pub fn new(auxiliary_data: Aux) -> Self {
        Self {
            converters: PreHashMap::with_capacity(128),
            auxiliary_data,
            callables: PreHashMap::default(),
            objects: PreHashMap::with_capacity(128),
            runtime_data: RuntimeData {
                memory_limit: 40000,
                memory: Vec::with_capacity(512),
                stack: ScalarStack::new(256),
                global_vars: Vec::with_capacity(128),
                call_stack: BoundedStack::new(256),
            },
            max_instr: 1000,
            _m: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.converters.clear();
        self.runtime_data.clear();
    }

    pub fn get_object_properties(&self, ptr: Pointer) -> Option<Box<dyn ObjectProperties>> {
        let object = self.objects.get(Key::from_u32(ptr.0))?;
        object.as_inner(self).ok()
    }

    pub fn read_var_by_name(&self, name: &str, vars: &Variables) -> Option<Scalar> {
        let varid = vars.ids.get(Key::from_str(name).ok()?)?;
        self.read_var(*varid)
    }

    #[inline]
    pub fn read_var(&self, name: VariableId) -> Option<Scalar> {
        self.runtime_data.global_vars.get(name.0 as usize).cloned()
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_instr = max_iter;
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

    /// ```
    /// use cao_lang::prelude::*;
    ///
    /// fn my_epic_func(vm: &mut Vm<()>, inp: i32) -> Result<(), ExecutionError> {
    ///     vm.stack_push(inp * 2);
    ///     Ok(())
    /// }
    ///
    /// let mut vm = Vm::new(());
    /// vm.register_function("epic", my_epic_func as VmFunction1<_, _>);
    /// ```
    pub fn register_function<S, C>(&mut self, name: S, f: C)
    where
        S: Into<String>,
        C: VmFunction<Aux> + 'static,
    {
        let name = name.into();
        let key = Key::from_str(name.as_str()).unwrap();
        self.callables.insert(key, Procedure::new(name, f));
    }

    pub fn get_value_in_place<T: DecodeInPlace<'a>>(
        &'a self,
        instr_ptr: Pointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        let object = self.objects.get(Key::from_u32(instr_ptr.0))?;

        self.runtime_data.get_value_in_place::<T>(object)
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, instr_ptr: Pointer) -> Option<T> {
        let object = self.objects.get(Key::from_u32(instr_ptr.0))?;
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
        let (handle, size) = self.runtime_data.write_to_memory(val)?;
        let object = Object {
            index: Some(handle),
            size: size as u32,
        };
        let key = Key::from_u32(handle.0);
        self.objects.insert(key, object);
        self.converters.insert(key, converter);

        self.stack_push(Scalar::Pointer(handle as Pointer))?;

        Ok(object)
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value<T: ByteEncodeProperties + ByteDecodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<Object, ExecutionError> {
        let (handle, size) = self.runtime_data.write_to_memory(val)?;
        let object = Object {
            index: Some(handle),
            size: size as u32,
        };
        let key = Key::from_u32(handle.0);
        self.objects.insert(key, object);
        self.converters.insert(key, |o: &Object, vm: &Vm<Aux>| {
            let res: T = vm.get_value(o.index.unwrap()).unwrap();
            Box::new(res)
        });

        self.stack_push(Scalar::Pointer(handle as Pointer))?;

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
    /// As such running non-compiler emitted programs is very un-safe
    pub fn run(&mut self, program: &CaoProgram) -> Result<i32, ExecutionError> {
        self.runtime_data
            .call_stack
            .push(CallFrame {
                instr_ptr: 0,
                stack_offset: 0,
            })
            .map_err(|_| ExecutionError::CallStackOverflow)?;
        let len = program.bytecode.len();
        let mut remaining_iters = self.max_instr;

        let mut instr_ptr = 0;
        while instr_ptr < len {
            remaining_iters -= 1;
            if remaining_iters <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr: u8 = unsafe { *program.bytecode.as_ptr().add(instr_ptr) };
            let instr: Instruction = unsafe { transmute(instr) };
            instr_ptr += 1;
            match instr {
                Instruction::GotoIfTrue => {
                    let condition = self.runtime_data.stack.pop();
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    debug_assert!(pos >= 0);
                    if condition.as_bool() {
                        instr_ptr = pos as usize;
                    }
                }
                Instruction::GotoIfFalse => {
                    let condition = self.runtime_data.stack.pop();
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    debug_assert!(pos >= 0);
                    if !condition.as_bool() {
                        instr_ptr = pos as usize;
                    }
                }
                Instruction::Goto => {
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    debug_assert!(pos >= 0);
                    instr_ptr = pos as usize;
                }
                Instruction::SwapLast => {
                    let b = pop_stack!(self);
                    let a = pop_stack!(self);
                    // we popped two values, we know that the stack has capacity for 2 ..
                    self.stack_push(b).unwrap();
                    self.stack_push(a).unwrap();
                }
                Instruction::ScalarNull => self.stack_push(Scalar::Null)?,
                Instruction::ClearStack => {
                    self.runtime_data.stack.clear_until(
                        self.runtime_data
                            .call_stack
                            .last()
                            .expect("No callframe available")
                            .stack_offset,
                    );
                }
                Instruction::SetLocalVar => {
                    instr_execution::set_local(self, &program.bytecode, &mut instr_ptr)?;
                }
                Instruction::ReadLocalVar => {
                    instr_execution::get_local(self, &program.bytecode, &mut instr_ptr)?;
                }
                Instruction::SetGlobalVar => {
                    instr_execution::instr_set_var(
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut instr_ptr,
                    )?;
                }
                Instruction::ReadGlobalVar => {
                    instr_execution::instr_read_var(
                        &mut self.runtime_data,
                        &mut instr_ptr,
                        &program,
                    )?;
                }
                Instruction::Pop => {
                    if !self.runtime_data.stack.is_empty() {
                        self.runtime_data.stack.pop();
                    }
                }
                Instruction::CallLane => {
                    instr_execution::instr_jump(&mut instr_ptr, program, &mut self.runtime_data)?;
                }
                Instruction::Return => {
                    instr_execution::instr_return(self, &mut instr_ptr)?;
                }
                Instruction::Exit => return instr_execution::instr_exit(&mut self.runtime_data),
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
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Floating(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarArray => instr_execution::instr_scalar_array(
                    &mut self.runtime_data,
                    &program.bytecode,
                    &mut instr_ptr,
                )?,
                Instruction::Not => {
                    let value = self.stack_pop();
                    let value = !value.as_bool();
                    self.stack_push(Scalar::Integer(value as i32))?;
                }
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
                Instruction::StringLiteral => {
                    instr_execution::instr_string_literal(self, &mut instr_ptr, &program)?
                }
                Instruction::Call => {
                    instr_execution::execute_call(self, &mut instr_ptr, &program.bytecode)?
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
