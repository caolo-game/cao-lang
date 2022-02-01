use std::convert::TryFrom;

use tracing::debug;

use crate::{
    bytecode::{decode_str, read_from_bytes, TriviallyEncodable},
    collections::key_map::Handle,
    procedures::ExecutionErrorPayload,
    program::CaoProgram,
    traits::MAX_STR_LEN,
    value::Value,
    VariableId,
};

use super::{
    runtime::{CallFrame, RuntimeData},
    Vm,
};

pub fn read_str<'a>(instr_ptr: &mut usize, program: &'a [u8]) -> Option<&'a str> {
    let p = *instr_ptr;
    let limit = program.len().min(p + MAX_STR_LEN);
    let (len, s): (_, &'a str) = decode_str(&program[p..limit])?;
    *instr_ptr += len;
    Some(s)
}

/// # Safety
///
/// Assumes that the underlying data is safely decodable to the given type
///
pub unsafe fn decode_value<T: TriviallyEncodable>(bytes: &[u8], instr_ptr: &mut usize) -> T {
    let (len, val) = read_from_bytes(&bytes[*instr_ptr..]).expect("Failed to read data");
    *instr_ptr += len;
    val
}

type ExecutionResult = Result<(), ExecutionErrorPayload>;

pub fn instr_read_var(
    runtime_data: &mut RuntimeData,
    instr_ptr: &mut usize,
    program: &CaoProgram,
) -> ExecutionResult {
    let VariableId(varid): VariableId = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let value = runtime_data
        .global_vars
        .get(varid as usize)
        .ok_or_else(|| {
            ExecutionErrorPayload::VarNotFound(
                program
                    .variables
                    .names
                    .get(&VariableId(varid))
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "<<<Unknown variable>>>".to_string()),
            )
        })?;
    runtime_data
        .stack
        .push(*value)
        .map_err(|_| ExecutionErrorPayload::Stackoverflow)?;
    Ok(())
}

pub fn instr_set_var(
    runtime_data: &mut RuntimeData,
    bytecode: &[u8],
    instr_ptr: &mut usize,
) -> ExecutionResult {
    let varname = unsafe { decode_value::<VariableId>(bytecode, instr_ptr) };
    let scalar = runtime_data.stack.pop();
    let varid = varname.0 as usize;
    if runtime_data.global_vars.len() <= varid {
        runtime_data.global_vars.resize(varid + 1, Value::Nil);
    }
    runtime_data.global_vars[varid] = scalar;
    Ok(())
}

pub fn instr_len<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let val = vm.stack_pop();
    let len = match val {
        Value::Nil => 0,
        Value::Integer(_) | Value::Floating(_) => 1,
        Value::String(s) => {
            let st = unsafe {
                s.get_str().ok_or_else(|| {
                    ExecutionErrorPayload::invalid_argument("String not found".to_string())
                })?
            };
            st.len() as i64
        }
        Value::Object(t) => {
            let t = unsafe { &*t };
            t.len() as i64
        }
    };
    vm.stack_push(len)?;
    Ok(())
}

pub fn instr_string_literal<T>(
    vm: &mut Vm<T>,
    instr_ptr: &mut usize,
    program: &CaoProgram,
) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let payload = read_str(&mut (handle as usize), program.data.as_slice())
        .ok_or_else(|| ExecutionErrorPayload::invalid_argument(None))?;

    let ptr = vm.init_string(payload)?;
    vm.stack_push(Value::String(ptr))?;

    Ok(())
}

pub fn instr_jump(
    instr_ptr: &mut usize,
    program: &CaoProgram,
    runtime_data: &mut RuntimeData,
) -> ExecutionResult {
    let label: Handle = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let argcount: u32 = unsafe { decode_value(&program.bytecode, instr_ptr) };

    // remember the location after this jump
    runtime_data
        .call_stack
        .last_mut()
        .expect("Call stack was empty")
        .instr_ptr = *instr_ptr;

    // init the new call frame
    runtime_data
        .call_stack
        .push(CallFrame {
            instr_ptr: *instr_ptr,
            stack_offset: runtime_data
                .stack
                .len()
                .checked_sub(argcount as usize)
                .ok_or(ExecutionErrorPayload::MissingArgument)?,
        })
        .map_err(|_| ExecutionErrorPayload::CallStackOverflow)?;

    // set the instr_ptr to the new lane's beginning
    *instr_ptr = program.labels.0.get(label).expect("Label not found").pos as usize;
    Ok(())
}

pub fn execute_call<T>(vm: &mut Vm<T>, instr_ptr: &mut usize, bytecode: &[u8]) -> ExecutionResult {
    let fun_hash: Handle = unsafe { decode_value(bytecode, instr_ptr) };
    let procedure = vm
        .callables
        .remove(fun_hash)
        .ok_or(ExecutionErrorPayload::ProcedureNotFound(fun_hash))?;
    let res = procedure
        .fun
        .call(vm)
        .map_err(|err| ExecutionErrorPayload::TaskFailure {
            name: procedure.name.clone(),
            error: Box::new(err),
        });
    //cleanup
    vm.callables
        .insert(fun_hash, procedure)
        .expect("fun re-insert");
    res
}

pub fn set_local<T>(vm: &mut Vm<T>, bytecode: &[u8], instr_ptr: &mut usize) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let offset = vm
        .runtime_data
        .call_stack
        .last()
        .expect("Call stack is emtpy")
        .stack_offset;
    let value = vm.runtime_data.stack.pop_w_offset(offset);
    vm.runtime_data
        .stack
        .set(offset + handle as usize, value)
        .map_err(|err| {
            ExecutionErrorPayload::VarNotFound(format!("Failed to set local variable: {}", err))
        })?;
    Ok(())
}

pub fn get_local<T>(vm: &mut Vm<T>, bytecode: &[u8], instr_ptr: &mut usize) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let offset = vm
        .runtime_data
        .call_stack
        .last()
        .expect("Call stack is emtpy")
        .stack_offset;
    let value = vm.runtime_data.stack.get(offset + handle as usize);
    vm.stack_push(value)?;
    Ok(())
}

pub fn instr_return<T>(vm: &mut Vm<T>, instr_ptr: &mut usize) -> ExecutionResult {
    // pop the current stack frame
    let value = match vm.runtime_data.call_stack.pop() {
        // return value
        Some(rt) => vm.runtime_data.stack.clear_until(rt.stack_offset),
        None => {
            return Err(ExecutionErrorPayload::BadReturn {
                reason: "Call stack is empty".to_string(),
            })
        }
    };
    // read the previous frame
    match vm.runtime_data.call_stack.last_mut() {
        Some(CallFrame { instr_ptr: ptr, .. }) => {
            *instr_ptr = *ptr;
        }
        None => {
            return Err(ExecutionErrorPayload::BadReturn {
                reason: "Failed to find return address".to_string(),
            });
        }
    }
    // push the return value
    vm.stack_push(value)?;
    Ok(())
}

pub fn begin_repeat<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let n = vm.runtime_data.stack.last();
    let n = i64::try_from(n).map_err(|_| {
        ExecutionErrorPayload::invalid_argument("Repeat input must be an integer".to_string())
    })?;
    if n < 0 {
        return Err(ExecutionErrorPayload::invalid_argument(
            "Repeat input must be non-negative".to_string(),
        ));
    }
    // init the loop counter
    vm.stack_push(0)?;
    Ok(())
}

/// return if the loop should continue
pub fn repeat<T>(vm: &mut Vm<T>) -> Result<bool, ExecutionErrorPayload> {
    let i = vm.stack_pop();
    let i = i64::try_from(i).expect("Repeat input `I` must be an integer");
    let n = vm.runtime_data.stack.last();
    let n = i64::try_from(n).expect("Repeat input `N` must be an integer");

    let should_continue = i < n;
    if should_continue {
        // restore the variable and add 1
        vm.stack_push(i + 1)?;
        // push the lane argument
        vm.stack_push(i)?;
    } else {
        // clean up
        vm.stack_pop(); // N
    }

    Ok(should_continue)
}

pub fn instr_copy_last<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let val = vm.runtime_data.stack.last();
    vm.runtime_data
        .stack
        .push(val)
        .map_err(|_| ExecutionErrorPayload::Stackoverflow)?;

    Ok(())
}

/// push `i=0` onto the stack
pub fn begin_for_each<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let item = vm.runtime_data.stack.last();
    // test if the input is a table
    let _item = vm.get_table(item)?;
    debug!("Starting for-each on table {:?}", _item);

    vm.stack_push(0)?; // i, this should be incremented by `for_each`

    Ok(())
}

/// Assumes that [begin_for_each](begin_for_each) was called once to set up the loop
///
/// Requires `i`, and the object to be on the stack.
///
/// Pushes the next key and the object onto the stack. Assumes that the lane takes these as
/// parameters.
pub fn for_each<T>(vm: &mut Vm<T>) -> Result<bool, ExecutionErrorPayload> {
    let i = vm.stack_pop();
    let obj_val = vm.runtime_data.stack.peek_last(0);

    let mut i = i64::try_from(i).expect("Repeat input #0 must be an integer");
    let obj = vm
        .get_table(obj_val)
        .expect("Repeat input #2 must be a table");

    debug_assert!(0 <= i, "for_each overflow");

    let n = obj.len() as i64;

    let should_continue = 0 <= i && i < n;
    if should_continue {
        let key = obj.iter().nth(i as usize).map(|(k, _)| k).ok_or_else(|| {
            ExecutionErrorPayload::invalid_argument(format!(
                "ForEach can not find the `i`th argument. i: {} n: {}\nDid you remove items during iteration?",
                i, n
            ))
        })?;
        i += 1;

        // restore the variable
        vm.stack_push(i)?;
        // push lane arguments
        vm.stack_push(key)?;
        vm.stack_push(obj_val)?;
    } else {
        vm.stack_pop(); // clean up the object
    }

    Ok(should_continue)
}
