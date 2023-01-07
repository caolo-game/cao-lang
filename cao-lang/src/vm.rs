//! Cao-Lang back-end
//!
//! Interprets the compiled output produced by the Cao-Lang compiler
mod instr_execution;
pub mod runtime;

#[cfg(test)]
mod tests;

use self::runtime::CallFrame;
use crate::{
    collections::handle_table::{Handle, HandleTable},
    instruction::Instruction,
    prelude::*,
    value::Value,
    VariableId,
};
use runtime::RuntimeData;
use std::{mem::transmute, str::FromStr};
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

    pub runtime_data: RuntimeData,

    callables: HandleTable<Procedure<Aux>>,
    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> Vm<'a, Aux> {
    pub fn new(auxiliary_data: Aux) -> Result<Self, ExecutionErrorPayload> {
        Ok(Self {
            auxiliary_data,
            callables: HandleTable::default(),
            runtime_data: RuntimeData::new(400 * 1024, 256, 256)?,
            max_instr: 1000,
            _m: Default::default(),
        })
    }

    /// Inserts the given value into the VM's runtime memory. Returns the inserted [[Value]]
    pub fn insert_value(&mut self, value: &OwnedValue) -> Result<Value, ExecutionErrorPayload> {
        let res = match value {
            OwnedValue::Nil => Value::Nil,
            OwnedValue::String(s) => {
                let res = self.init_string(s.as_str())?;
                Value::String(res)
            }
            OwnedValue::Object(o) => {
                let mut res = self.init_table()?;
                for OwnedEntry { key, value } in o.iter() {
                    let key = self.insert_value(key)?;
                    let value = self.insert_value(value)?;
                    unsafe {
                        res.as_mut().insert(key, value)?;
                    }
                }
                Value::Object(res.as_ptr())
            }
            OwnedValue::Integer(x) => Value::Integer(*x),
            OwnedValue::Real(x) => Value::Real(*x),
            OwnedValue::Function { hash, arity } => Value::Function {
                hash: *hash,
                arity: *arity,
            },
        };
        Ok(res)
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
    pub fn register_function<S, C>(&mut self, name: S, f: C)
    where
        S: Into<String>,
        C: VmFunction<Aux> + 'static,
    {
        let name = name.into();
        let key = Handle::from_str(name.as_str()).unwrap();
        self.callables
            .insert(key, Procedure::new(name, f))
            .expect("failed to insert new function");
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

    pub fn get_table(&self, value: Value) -> Result<&FieldTable, ExecutionErrorPayload> {
        let res = match value {
            Value::Object(o) => unsafe { &*o },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionErrorPayload::invalid_argument(
                    "Input must be an Object".to_string(),
                ));
            }
        };
        Ok(res)
    }

    pub fn get_table_mut(&self, value: Value) -> Result<&mut FieldTable, ExecutionErrorPayload> {
        let res = match value {
            Value::Object(o) => unsafe { &mut *o },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionErrorPayload::invalid_argument(
                    "GetProperty input must be an Object".to_string(),
                ));
            }
        };
        Ok(res)
    }

    /// Initializes a new FieldTable in this VM instance
    #[inline]
    pub fn init_table(&mut self) -> Result<std::ptr::NonNull<FieldTable>, ExecutionErrorPayload> {
        self.runtime_data.init_table()
    }

    /// Initializes a new string owned by this VM instance
    pub fn init_string(&mut self, payload: &str) -> Result<StrPointer, ExecutionErrorPayload> {
        unsafe {
            let layout = std::alloc::Layout::from_size_align(4 + payload.len(), 4).unwrap();
            let mut ptr = self
                .runtime_data
                .memory
                .alloc(layout)
                .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;

            let result: *mut u8 = ptr.as_mut();
            std::ptr::write(result as *mut u32, payload.len() as u32);
            std::ptr::copy(payload.as_ptr(), result.add(4), payload.len());

            Ok(StrPointer(ptr.as_ptr()))
        }
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is very un-safe
    #[inline(never)]
    pub fn run(&mut self, program: &CaoCompiledProgram) -> ExecutionResult<()> {
        self.runtime_data
            .call_stack
            .push(CallFrame {
                src_instr_ptr: 0,
                dst_instr_ptr: 0,
                stack_offset: 0,
            })
            .map_err(|_| ExecutionErrorPayload::CallStackOverflow)
            .map_err(|pl| ExecutionError::new(pl, Vec::new()))?;

        let len = program.bytecode.len();
        let mut remaining_iters = self.max_instr;
        let mut instr_ptr = 0;
        let bytecode_ptr = program.bytecode.as_ptr();

        let payload_to_error =
            |err, instr_ptr, stack: &crate::collections::bounded_stack::BoundedStack<CallFrame>| {
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

        while instr_ptr < len {
            remaining_iters -= 1;
            if remaining_iters == 0 {
                return Err(payload_to_error(
                    ExecutionErrorPayload::Timeout,
                    instr_ptr,
                    &self.runtime_data.call_stack,
                ));
            }
            let instr: u8 = unsafe { *bytecode_ptr.add(instr_ptr) };
            let instr: Instruction = unsafe { transmute(instr) };
            let src_ptr = instr_ptr;
            instr_ptr += 1;
            debug!(
                "Executing: {:?} instr_ptr: {} Stack: {}",
                instr, instr_ptr, self.runtime_data.value_stack
            );
            match instr {
                Instruction::InitTable => {
                    let res = self.init_table().map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    self.stack_push(Value::Object(res.as_ptr()))
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::GetProperty => {
                    let key = self.stack_pop();
                    let instance = self.stack_pop();
                    let table = self.get_table(instance).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    let result = table.get(&key).copied().unwrap_or(Value::Nil);
                    self.stack_push(result).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::SetProperty => {
                    let [value, key, instance] = self.runtime_data.value_stack.pop_n::<3>();
                    let table = self.get_table_mut(instance).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                    table
                        .insert(key, value)
                        .map_err(|err| {
                            debug!("Failed to insert value {:?}", err);
                            ExecutionErrorPayload::OutOfMemory
                        })
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::BeginForEach => {
                    instr_execution::begin_for_each(self, &program.bytecode, &mut instr_ptr)
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::ForEach => {
                    instr_execution::for_each(self, &program.bytecode, &mut instr_ptr).map_err(
                        |err| payload_to_error(err, instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::GotoIfTrue => {
                    let condition = self.runtime_data.value_stack.pop();
                    let pos: i32 =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    debug_assert!(pos >= 0);
                    if condition.as_bool() {
                        instr_ptr = pos as usize;
                    }
                }
                Instruction::GotoIfFalse => {
                    let condition = self.runtime_data.value_stack.pop();
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
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    // we popped two values, we know that the stack has capacity for 2 ..
                    self.stack_push(b).unwrap();
                    self.stack_push(a).unwrap();
                }
                Instruction::ScalarNil => self.stack_push(Value::Nil).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::ClearStack => {
                    self.runtime_data.value_stack.clear_until(
                        self.runtime_data
                            .call_stack
                            .last()
                            .expect("No callframe available")
                            .stack_offset as usize,
                    );
                }
                Instruction::SetLocalVar => {
                    instr_execution::set_local(self, &program.bytecode, &mut instr_ptr).map_err(
                        |err| payload_to_error(err, instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::ReadLocalVar => {
                    instr_execution::get_local(self, &program.bytecode, &mut instr_ptr).map_err(
                        |err| payload_to_error(err, instr_ptr, &self.runtime_data.call_stack),
                    )?;
                }
                Instruction::SetGlobalVar => {
                    instr_execution::instr_set_var(
                        &mut self.runtime_data,
                        &program.bytecode,
                        &mut instr_ptr,
                    )
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::ReadGlobalVar => {
                    instr_execution::instr_read_var(
                        &mut self.runtime_data,
                        &mut instr_ptr,
                        program,
                    )
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::Pop => {
                    self.stack_pop();
                }
                Instruction::CallLane => {
                    instr_execution::instr_jump(
                        src_ptr,
                        &mut instr_ptr,
                        program,
                        &mut self.runtime_data,
                    )
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::Return => {
                    instr_execution::instr_return(self, &mut instr_ptr).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::Exit => return Ok(()),
                Instruction::CopyLast => {
                    instr_execution::instr_copy_last(self).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?;
                }
                Instruction::FunctionPointer => {
                    self.runtime_data
                        .value_stack
                        .push(Value::Function {
                            hash: unsafe {
                                instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                            },
                            arity: unsafe {
                                instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                            },
                        })
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .value_stack
                        .push(Value::Integer(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .value_stack
                        .push(Value::Real(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionErrorPayload::Stackoverflow)
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::Not => {
                    let value = self.stack_pop();
                    let value = !value.as_bool();
                    self.stack_push(Value::Integer(value as i64))
                        .map_err(|err| {
                            payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                        })?;
                }
                Instruction::And => self
                    .binary_op(|a, b| Value::from(a.as_bool() && b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Or => self
                    .binary_op(|a, b| Value::from(a.as_bool() || b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Xor => self
                    .binary_op(|a, b| Value::from(a.as_bool() ^ b.as_bool()))
                    .map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?,
                Instruction::Add => self.binary_op(|a, b| a + b).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Sub => self.binary_op(|a, b| a - b).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Mul => self.binary_op(|a, b| a * b).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Div => self.binary_op(|a, b| a / b).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::Equals => self.binary_op(|a, b| (a == b).into()).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::NotEquals => {
                    self.binary_op(|a, b| (a != b).into()).map_err(|err| {
                        payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                    })?
                }
                Instruction::Less => self.binary_op(|a, b| (a < b).into()).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::LessOrEq => self.binary_op(|a, b| (a <= b).into()).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
                Instruction::StringLiteral => {
                    instr_execution::instr_string_literal(self, &mut instr_ptr, program).map_err(
                        |err| payload_to_error(err, instr_ptr, &self.runtime_data.call_stack),
                    )?
                }
                Instruction::Call => {
                    instr_execution::execute_call(self, &mut instr_ptr, &program.bytecode).map_err(
                        |err| payload_to_error(err, instr_ptr, &self.runtime_data.call_stack),
                    )?
                }
                Instruction::Len => instr_execution::instr_len(self).map_err(|err| {
                    payload_to_error(err, instr_ptr, &self.runtime_data.call_stack)
                })?,
            }
        }

        Err(payload_to_error(
            ExecutionErrorPayload::UnexpectedEndOfInput,
            instr_ptr,
            &self.runtime_data.call_stack,
        ))
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
