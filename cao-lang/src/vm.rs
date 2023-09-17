//! Cao-Lang back-end
//!
//! Interprets the compiled output produced by the Cao-Lang compiler
mod instr_execution;
pub mod runtime;

#[cfg(test)]
mod tests;

use self::runtime::{
    cao_lang_object::{CaoLangObjectBody, ObjectGcGuard},
    CallFrame,
};
use crate::{
    collections::handle_table::{Handle, HandleTable},
    instruction::Instruction,
    prelude::*,
    stdlib,
    value::Value,
    vm::runtime::cao_lang_function::CaoLangClosure,
    VariableId,
};
use runtime::RuntimeData;
use std::{mem::transmute, ops::DerefMut, pin::Pin, str::FromStr};
use tracing::debug;

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary runtime structure passed to custom functions.
pub struct Vm<'a, Aux = ()>
where
    Aux: 'a,
{
    pub auxiliary_data: Aux,
    /// Number of instructions `run` will execute before returning Timeout
    pub max_instr: u64,
    pub remaining_iters: u64,

    pub runtime_data: Pin<Box<RuntimeData>>,

    callables: HandleTable<Procedure<Aux>>,
    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> Vm<'a, Aux> {
    pub fn new(auxiliary_data: Aux) -> Result<Self, ExecutionErrorPayload>
    where
        Aux: 'static,
    {
        let mut vm = Self {
            auxiliary_data,
            callables: HandleTable::default(),
            runtime_data: RuntimeData::new(400 * 1024, 256, 256)?,
            max_instr: 1000,
            remaining_iters: 0,
            _m: Default::default(),
        };
        vm.register_native_stdlib().unwrap();
        Ok(vm)
    }

    pub fn register_native_stdlib(&mut self) -> Result<(), ExecutionErrorPayload>
    where
        Aux: 'static,
    {
        self._register_native_function("__min", into_f2(stdlib::native_minmax::<Aux, true>))?;
        self._register_native_function("__max", into_f2(stdlib::native_minmax::<Aux, false>))?;
        self._register_native_function("__sort", into_f2(stdlib::native_sorted::<Aux>))?;
        Ok(())
    }

    /// Inserts the given value into the VM's runtime memory. Returns the inserted [[Value]]
    pub fn insert_value(&mut self, value: &OwnedValue) -> Result<Value, ExecutionErrorPayload> {
        let res = match value {
            OwnedValue::Nil => Value::Nil,
            OwnedValue::String(s) => {
                let res = self.init_string(s.as_str())?;
                Value::Object(res.0)
            }
            OwnedValue::Table(o) => {
                let mut res = self.init_table()?;
                let table = res.deref_mut().as_table_mut().unwrap();
                for OwnedEntry { key, value } in o.iter() {
                    let key = self.insert_value(key)?;
                    let value = self.insert_value(value)?;
                    table.insert(key, value)?;
                }
                Value::Object(res.0)
            }
            OwnedValue::Integer(x) => Value::Integer(*x),
            OwnedValue::Real(x) => Value::Real(*x),
        };
        Ok(res)
    }

    pub fn init_native_function(
        &mut self,
        handle: Handle,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_native_function(handle)
    }

    pub fn init_function(
        &mut self,
        handle: Handle,
        arity: u32,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_function(handle, arity)
    }

    pub fn init_closure(
        &mut self,
        handle: Handle,
        arity: u32,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_closure(handle, arity)
    }

    pub fn init_upvalue(
        &mut self,
        location: *mut Value,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_upvalue(location)
    }

    pub fn clear(&mut self) {
        self.runtime_data.clear();
    }

    pub fn read_var_by_name(&self, name: &str, vars: &Variables) -> Option<Value> {
        let varid = vars.ids.get(Handle::from_str(name).ok()?)?;
        self.read_var(*varid)
    }

    #[inline]
    pub fn read_var(&self, name: VariableId) -> Option<Value> {
        self.runtime_data.global_vars.get(name.0 as usize).cloned()
    }

    #[must_use]
    pub fn with_max_iter(mut self, max_iter: u64) -> Self {
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

    /// Register a native function for use by Cao-Lang programs
    ///
    pub fn register_native_function<S, C>(
        &mut self,
        name: S,
        f: C,
    ) -> Result<(), ExecutionErrorPayload>
    where
        S: AsRef<str>,
        C: VmFunction<Aux> + 'static,
    {
        if name.as_ref().starts_with("__") {
            return Err(ExecutionErrorPayload::invalid_argument(
                "Native function name may not begin with __",
            ));
        }
        self._register_native_function(name, f)
    }

    fn _register_native_function<S, C>(
        &mut self,
        name: S,
        f: C,
    ) -> Result<(), ExecutionErrorPayload>
    where
        S: AsRef<str>,
        C: VmFunction<Aux> + 'static,
    {
        let key = Handle::from_str(name.as_ref()).unwrap();
        let name = self.init_string(name.as_ref())?;
        self.callables
            .insert(
                key,
                Procedure {
                    name: name.0,
                    fun: std::rc::Rc::new(f),
                },
            )
            .map_err(|_| ExecutionErrorPayload::OutOfMemory)
            .map(drop)
    }

    #[inline]
    pub fn stack_push<S>(&mut self, value: S) -> Result<(), ExecutionErrorPayload>
    where
        S: Into<Value>,
    {
        self.runtime_data
            .value_stack
            .push(value.into())
            .map_err(|_| ExecutionErrorPayload::Stackoverflow)?;
        Ok(())
    }

    #[inline]
    pub fn stack_pop(&mut self) -> Value {
        self.runtime_data.value_stack.pop()
    }

    pub fn get_table(&self, value: Value) -> Result<&CaoLangTable, ExecutionErrorPayload> {
        let res = match value {
            Value::Object(o) => unsafe {
                o.as_ref()
                    .as_table()
                    .ok_or_else(|| ExecutionErrorPayload::invalid_argument("Expected Table"))?
            },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionErrorPayload::invalid_argument("Expected Table"));
            }
        };
        Ok(res)
    }

    pub fn get_table_mut(&self, value: Value) -> Result<&mut CaoLangTable, ExecutionErrorPayload> {
        let res = match value {
            Value::Object(mut o) => unsafe {
                o.as_mut()
                    .as_table_mut()
                    .ok_or_else(|| ExecutionErrorPayload::invalid_argument("Expected Table"))?
            },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionErrorPayload::invalid_argument("Expected Table"));
            }
        };
        Ok(res)
    }

    /// Initializes a new FieldTable in this VM instance
    #[inline]
    pub fn init_table(&mut self) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_table()
    }

    /// Initializes a new string owned by this VM instance
    pub fn init_string(&mut self, payload: &str) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        self.runtime_data.init_string(payload)
    }

    /// Panics if no current program has been set
    pub fn run_function(&mut self, val: Value) -> Result<Value, ExecutionErrorPayload> {
        let Value::Object(obj) = val else {
            return Err(ExecutionErrorPayload::invalid_argument(
                "Expected a function object argument",
            ));
        };
        let arity;
        let label;
        let mut closure: *mut CaoLangClosure = std::ptr::null_mut();
        unsafe {
            match &obj.as_ref().body {
                CaoLangObjectBody::Closure(c) => {
                    arity = c.function.arity;
                    label = c.function.handle;
                    closure = (c as *const CaoLangClosure).cast_mut();
                }
                CaoLangObjectBody::Function(f) => {
                    arity = f.arity;
                    label = f.handle;
                }
                CaoLangObjectBody::NativeFunction(f) => {
                    instr_execution::call_native(self, f.handle)?;
                    return Ok(self.stack_pop());
                }
                _ => {
                    return Err(ExecutionErrorPayload::invalid_argument(format!(
                        "Expected a function object argument, instead got: {}",
                        obj.as_ref().type_name()
                    )));
                }
            }
        }
        let program: &CaoCompiledProgram = unsafe {
            let program = self.runtime_data.current_program;
            assert!(!program.is_null());
            &*program
        };
        debug_assert!(!program.bytecode.is_empty());

        let func = program
            .labels
            .0
            .get(label)
            .ok_or_else(|| ExecutionErrorPayload::ProcedureNotFound(label))?;

        let src = func.pos;
        let end = program.bytecode.len() - 1;
        let len = self.runtime_data.value_stack.len() as u32;

        // a function call needs 2 stack frames, 1 for the current scope, another for the return
        // address
        //
        // the first one will be used as a trap, to exit the program,
        // the second one is the actual callframe of the function
        for _ in 0..2 {
            self.runtime_data
                .call_stack
                .push(CallFrame {
                    src_instr_ptr: src,
                    dst_instr_ptr: end as u32,
                    stack_offset: len
                        .checked_sub(arity)
                        .ok_or(ExecutionErrorPayload::MissingArgument)?
                        as u32,
                    closure,
                })
                .map_err(|_| ExecutionErrorPayload::CallStackOverflow)?;
        }

        let mut instr_ptr = src as usize;
        self._run(&mut instr_ptr).map_err(|err| err.payload)?;
        // pop the trap callframe
        self.runtime_data.call_stack.pop();
        Ok(self.stack_pop())
    }

    fn _run(&mut self, instr_ptr: &mut usize) -> ExecutionResult<()> {
        let program: &CaoCompiledProgram = unsafe {
            let program = self.runtime_data.current_program;
            assert!(!program.is_null());
            &*program
        };
        let len = program.bytecode.len();
        // FIXME: should store in VM
        let mut remaining_iters = self.max_instr;
        let bytecode_ptr = program.bytecode.as_ptr();
        let payload_to_error =
            |err,
             instr_ptr: usize,
             stack: &crate::collections::bounded_stack::BoundedStack<CallFrame>| {
                let mut trace = Vec::with_capacity(stack.len() + 1);
                if let Some(t) = program.trace.get(&(instr_ptr as u32)).cloned() {
                    trace.push(t);
                }
                for t in stack.iter_backwards() {
                    if let Some(t) = program.trace.get(&t.src_instr_ptr) {
                        trace.push(t.clone())
                    }
                }
                ExecutionError::new(err, trace)
            };

        while *instr_ptr < len {
            remaining_iters -= 1;
            if remaining_iters == 0 {
                return Err(payload_to_error(
                    ExecutionErrorPayload::Timeout,
                    *instr_ptr,
                    &self.runtime_data.call_stack,
                ));
            }
            let instr: u8 = unsafe { *bytecode_ptr.add(*instr_ptr) };
            let instr: Instruction = unsafe { transmute(instr) };
            let src_ptr = *instr_ptr;
            *instr_ptr += 1;
            debug!("Executing: {instr:?} instr_ptr: {instr_ptr}");
            match instr {
                Instruction::InitTable => {
                    let res = self.init_table().map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    self.stack_push(Value::Object(res.0)).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::GetProperty => {
                    let key = self.stack_pop();
                    let instance = self.stack_pop();
                    let table = self.get_table(instance).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    let result = table.get(&key).copied().unwrap_or(Value::Nil);
                    self.stack_push(result).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::SetProperty => {
                    let [key, instance, value] = self.runtime_data.value_stack.pop_n::<3>();
                    let table = self.get_table_mut(instance).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    table
                        .insert(key, value)
                        .map_err(|err| {
                            debug!("Failed to insert value {:?}", err);
                            ExecutionErrorPayload::OutOfMemory
                        })
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::BeginForEach => {
                    instr_execution::begin_for_each(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::ForEach => {
                    instr_execution::for_each(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::GotoIfTrue => {
                    let condition = self.runtime_data.value_stack.pop();
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    debug_assert!(pos >= 0);
                    if condition.as_bool() {
                        *instr_ptr = pos as usize;
                    }
                }
                Instruction::GotoIfFalse => {
                    let condition = self.runtime_data.value_stack.pop();
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    debug_assert!(pos >= 0);
                    if !condition.as_bool() {
                        *instr_ptr = pos as usize;
                    }
                }
                Instruction::Goto => {
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    debug_assert!(pos >= 0);
                    *instr_ptr = pos as usize;
                }
                Instruction::SwapLast => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    // we popped two values, we know that the stack has capacity for 2 ..
                    self.stack_push(b).unwrap();
                    self.stack_push(a).unwrap();
                }
                Instruction::ScalarNil => self.stack_push(Value::Nil).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::ClearStack => {
                    let offset = self
                        .runtime_data
                        .call_stack
                        .last()
                        .expect("No callframe available")
                        .stack_offset as usize;
                    self.runtime_data.value_stack.clear_until(offset);
                }
                Instruction::SetLocalVar => {
                    instr_execution::set_local(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::ReadLocalVar => {
                    instr_execution::get_local(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::SetGlobalVar => {
                    instr_execution::instr_set_var(
                        &mut self.runtime_data,
                        &program.bytecode,
                        instr_ptr,
                    )
                    .map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::ReadGlobalVar => {
                    instr_execution::instr_read_var(&mut self.runtime_data, instr_ptr, program)
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::Pop => {
                    self.stack_pop();
                }
                Instruction::CallFunction => {
                    instr_execution::instr_call_function(src_ptr, instr_ptr, program, self)
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::Return => {
                    instr_execution::instr_return(self, instr_ptr).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::Exit => return Ok(()),
                Instruction::CopyLast => {
                    instr_execution::instr_copy_last(self).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::NativeFunctionPointer => {
                    let handle: u32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    let fun_name =
                        instr_execution::read_str(&mut (handle as usize), program.data.as_slice())
                            .ok_or(ExecutionErrorPayload::InvalidArgument { context: None })
                            .map_err(|err| {
                                payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                            })?;
                    let handle = Handle::from_str(fun_name).unwrap();
                    let obj = self.init_native_function(handle).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    let val = Value::Object(obj.0);
                    self.runtime_data
                        .value_stack
                        .push(val)
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            // free the object on Stackoverflow
                            self.runtime_data.free_object(obj.0);
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::FunctionPointer => {
                    let hash: Handle =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    let arity: u32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };

                    let obj = self.init_function(hash, arity).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;

                    let val = Value::Object(obj.0);

                    self.runtime_data
                        .value_stack
                        .push(val)
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            // free the object on Stackoverflow
                            self.runtime_data.free_object(obj.0);
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::Closure => {
                    let hash: Handle =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };
                    let arity: u32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, instr_ptr) };

                    let obj = self.init_closure(hash, arity).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;

                    let val = Value::Object(obj.0);

                    self.runtime_data
                        .value_stack
                        .push(val)
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            // free the object on Stackoverflow
                            self.runtime_data.free_object(obj.0);
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .value_stack
                        .push(Value::Integer(unsafe {
                            instr_execution::decode_value(&program.bytecode, instr_ptr)
                        }))
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .value_stack
                        .push(Value::Real(unsafe {
                            instr_execution::decode_value(&program.bytecode, instr_ptr)
                        }))
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::Not => {
                    let value = self.stack_pop();
                    let value = !value.as_bool();
                    self.stack_push(Value::Integer(value as i64))
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::And => self
                    .binary_op(|a, b| Value::from(a.as_bool() && b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Or => self
                    .binary_op(|a, b| Value::from(a.as_bool() || b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Xor => self
                    .binary_op(|a, b| Value::from(a.as_bool() ^ b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Add => self.binary_op(|a, b| a + b).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Sub => self.binary_op(|a, b| a - b).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Mul => self.binary_op(|a, b| a * b).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Div => self.binary_op(|a, b| a / b).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Equals => self.binary_op(|a, b| (a == b).into()).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::NotEquals => {
                    self.binary_op(|a, b| (a != b).into()).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?
                }
                Instruction::Less => self.binary_op(|a, b| (a < b).into()).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::LessOrEq => self.binary_op(|a, b| (a <= b).into()).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::StringLiteral => instr_execution::instr_string_literal(
                    self, instr_ptr, program,
                )
                .map_err(|err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack))?,
                Instruction::CallNative => {
                    instr_execution::execute_call_native(self, instr_ptr, &program.bytecode)
                        .map_err(|err| {
                            payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                        })?
                }
                Instruction::Len => instr_execution::instr_len(self).map_err(|err| {
                    payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::NthRow => {
                    let [i, instance] = self.runtime_data.value_stack.pop_n::<2>();
                    let table = self.get_table_mut(instance).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    let i = i.as_int().ok_or_else(|| {
                        payload_to_error(
                            ExecutionErrorPayload::invalid_argument(
                                "Input must be an integer".to_string(),
                            ),
                            *instr_ptr,
                            &self.runtime_data.call_stack,
                        )
                    })?;
                    if i < 0 {
                        return Err(payload_to_error(
                            ExecutionErrorPayload::invalid_argument(
                                "Input must be non-negative".to_string(),
                            ),
                            *instr_ptr,
                            &self.runtime_data.call_stack,
                        ));
                    }
                    let key = table.nth_key(i as usize);
                    let value = table.get(&key).copied().unwrap_or(Value::Nil);

                    debug!(
                        i = i,
                        key = tracing::field::debug(key),
                        value = tracing::field::debug(value),
                        table = tracing::field::debug(instance),
                        "Getting row of table"
                    );

                    (|| {
                        let mut row = self.init_table()?;
                        let row_table = row.as_table_mut().unwrap();
                        let k = self.init_string("key")?;
                        let v = self.init_string("value")?;
                        row_table.insert(Value::Object(k.0), key)?;
                        row_table.insert(Value::Object(v.0), value)?;
                        self.stack_push(Value::Object(row.0))?;
                        Ok(())
                    })()
                    .map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::AppendTable => {
                    let instance = self.stack_pop();
                    let value = self.stack_pop();
                    let table = self.get_table_mut(instance).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    table.append(value).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }

                Instruction::PopTable => {
                    let instance = self.stack_pop();
                    let table = self.get_table_mut(instance).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    let value = table.pop().map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    self.stack_push(value).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::SetUpvalue => {
                    instr_execution::write_upvalue(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::ReadUpvalue => {
                    instr_execution::read_upvalue(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::RegisterUpvalue => {
                    instr_execution::register_upvalue(self, &program.bytecode, instr_ptr).map_err(
                        |err| payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::CloseUpvalue => {
                    instr_execution::close_upvalues(self).map_err(|err| {
                        payload_to_error(err, *instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
            }
            debug!("Stack: {}", self.runtime_data.value_stack);
        }

        Err(payload_to_error(
            ExecutionErrorPayload::UnexpectedEndOfInput,
            *instr_ptr,
            &self.runtime_data.call_stack,
        ))
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is very un-safe
    pub fn run(&mut self, program: &CaoCompiledProgram) -> ExecutionResult<()> {
        self.runtime_data.current_program = program as *const _;
        self.runtime_data
            .call_stack
            .push(CallFrame {
                src_instr_ptr: 0,
                dst_instr_ptr: 0,
                stack_offset: 0,
                closure: std::ptr::null_mut(),
            })
            .map_err(|_| ExecutionErrorPayload::CallStackOverflow)
            .map_err(|pl| ExecutionError::new(pl, Default::default()))?;

        self.remaining_iters = self.max_instr;
        let mut instr_ptr = 0;
        let result = self._run(&mut instr_ptr);
        self.runtime_data.current_program = std::ptr::null();
        result
    }

    #[inline]
    fn binary_op(&mut self, op: fn(Value, Value) -> Value) -> Result<(), ExecutionErrorPayload> {
        let b = self.stack_pop();
        let a = self.stack_pop();

        self.runtime_data
            .value_stack
            .push(op(a, b))
            .map_err(|_| ExecutionErrorPayload::Stackoverflow)?;
        Ok(())
    }
}
