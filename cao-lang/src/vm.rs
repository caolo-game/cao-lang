//! Cao-Lang back-end
//!
//! Interprets the compiled output produced by the Cao-Lang compiler
mod instr_execution;
pub mod runtime;

#[cfg(test)]
mod tests;

use crate::VariableId;
use crate::{binary_compare, pop_stack};
use crate::{
    collections::key_map::{Key, KeyMap},
    instruction::Instruction,
    prelude::*,
};
use crate::{value::Value, StrPointer};
use runtime::RuntimeData;
use std::mem::transmute;
use std::str::FromStr;

use self::runtime::CallFrame;
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

    callables: KeyMap<Procedure<Aux>>,
    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> Vm<'a, Aux> {
    pub fn new(auxiliary_data: Aux) -> Result<Self, ExecutionError> {
        Ok(Self {
            auxiliary_data,
            callables: KeyMap::default(),
            runtime_data: RuntimeData::new(400 * 1024, 256, 256)?,
            max_instr: 1000,
            _m: Default::default(),
        })
    }

    pub fn clear(&mut self) {
        self.runtime_data.clear();
    }

    pub fn read_var_by_name(&self, name: &str, vars: &Variables) -> Option<Value> {
        let varid = vars.ids.get(Key::from_str(name).ok()?)?;
        self.read_var(*varid)
    }

    #[inline]
    pub fn read_var(&self, name: VariableId) -> Option<Value> {
        self.runtime_data.global_vars.get(name.0 as usize).cloned()
    }

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

    /// # Safety
    ///
    /// Must be called with ptr obtained from a `string_literal` instruction, before the last `clear`!
    ///
    /// # Return value
    ///
    /// Returns None if the underlying string is not valid utf8
    #[inline]
    pub unsafe fn get_str(&self, StrPointer(ptr): StrPointer) -> Option<&str> {
        let len = *(ptr as *const u32);
        let ptr = ptr.add(4);
        std::str::from_utf8(std::slice::from_raw_parts(ptr, len as usize)).ok()
    }

    /// Register a native function for use by Cao-Lang programs
    ///
    pub fn register_function<'b, S, C>(&mut self, name: S, f: C)
    where
        S: Into<&'b str>,
        C: VmFunction<Aux> + 'static,
    {
        let name = name.into();
        let key = Key::from_str(name).unwrap();
        self.callables
            .insert(key, Procedure::new(name, f))
            .expect("failed to insert new function");
    }

    #[inline]
    pub fn stack_push<S>(&mut self, value: S) -> Result<(), ExecutionError>
    where
        S: Into<Value>,
    {
        self.runtime_data
            .stack
            .push(value.into())
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    #[inline]
    pub fn stack_pop(&mut self) -> Value {
        self.runtime_data.stack.pop()
    }

    pub fn get_table(&self, value: Value) -> Result<&FieldTable, ExecutionError> {
        let res = match value {
            Value::Object(o) => unsafe { &*o },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionError::invalid_argument(
                    "GetProperty input must be an Object".to_string(),
                ));
            }
        };
        Ok(res)
    }

    pub fn get_table_mut(&self, value: Value) -> Result<&mut FieldTable, ExecutionError> {
        let res = match value {
            Value::Object(o) => unsafe { &mut *o },
            _ => {
                debug!("Got {:?} instead of object", value);
                return Err(ExecutionError::invalid_argument(
                    "GetProperty input must be an Object".to_string(),
                ));
            }
        };
        Ok(res)
    }

    /// Initializes a new FieldTable in this VM instance
    pub fn init_table(&mut self) -> Result<std::ptr::NonNull<FieldTable>, ExecutionError> {
        self.runtime_data.init_table()
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is very un-safe
    pub fn run(&mut self, program: &CaoProgram) -> Result<(), ExecutionError> {
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
            if remaining_iters == 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr: u8 = unsafe { *program.bytecode.as_ptr().add(instr_ptr) };
            let instr: Instruction = unsafe { transmute(instr) };
            instr_ptr += 1;
            debug!(
                "Executing: {:?} instr_ptr: {} Stack: {}",
                instr, instr_ptr, self.runtime_data.stack
            );
            match instr {
                Instruction::InitTable => {
                    let res = self.init_table()?;
                    self.stack_push(Value::Object(res.as_ptr()))?;
                }
                Instruction::GetProperty => {
                    let handle: Key =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    let instance = self.stack_pop();
                    let table = self.get_table(instance)?;
                    let result = table.get(handle).copied().unwrap_or(Value::Nil);
                    self.stack_push(result)?;
                }
                Instruction::SetProperty => {
                    let handle: Key =
                        unsafe { instr_execution::decode_value(&program.bytecode, &mut instr_ptr) };
                    let instance = self.stack_pop();
                    let value = self.stack_pop();
                    let table = self.get_table_mut(instance)?;
                    table.insert(handle, value).map_err(|err| {
                        debug!("Failed to insert value {:?}", err);
                        ExecutionError::OutOfMemory
                    })?;
                }
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
                Instruction::ScalarNil => self.stack_push(Value::Nil)?,
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
                Instruction::Exit => return Ok(()),
                Instruction::CopyLast => {
                    let val = self.runtime_data.stack.last();
                    self.runtime_data
                        .stack
                        .push(val)
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::Pass => {}
                Instruction::ScalarInt => {
                    self.runtime_data
                        .stack
                        .push(Value::Integer(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .stack
                        .push(Value::Floating(unsafe {
                            instr_execution::decode_value(&program.bytecode, &mut instr_ptr)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::Not => {
                    let value = self.stack_pop();
                    let value = !value.as_bool();
                    self.stack_push(Value::Integer(value as i64))?;
                }
                Instruction::And => {
                    self.binary_op(|a, b| Value::from(a.as_bool() && b.as_bool()))?
                }
                Instruction::Or => {
                    self.binary_op(|a, b| Value::from(a.as_bool() || b.as_bool()))?
                }
                Instruction::Xor => {
                    self.binary_op(|a, b| Value::from(a.as_bool() ^ b.as_bool()))?
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
        F: Fn(Value, Value) -> Value,
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
