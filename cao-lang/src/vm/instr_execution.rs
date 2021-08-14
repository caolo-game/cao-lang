use std::alloc::Layout;
use std::convert::TryFrom;

use crate::{
    bytecode::{decode_str, read_from_bytes, TriviallyEncodable},
    collections::key_map::Key,
    procedures::ExecutionError,
    procedures::ExecutionResult,
    program::CaoProgram,
    traits::MAX_STR_LEN,
    value::Value,
    StrPointer, VariableId,
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
            ExecutionError::VarNotFound(
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
        .map_err(|_| ExecutionError::Stackoverflow)?;
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
                vm.get_str(s).ok_or_else(|| {
                    ExecutionError::invalid_argument("String not found".to_string())
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
        .ok_or_else(|| ExecutionError::invalid_argument(None))?;

    unsafe {
        let layout = Layout::from_size_align(4 + payload.len(), 4).unwrap();
        let mut ptr = vm
            .runtime_data
            .memory
            .alloc(layout)
            .map_err(|_| ExecutionError::OutOfMemory)?;

        let result: *mut u8 = ptr.as_mut();
        std::ptr::write(result as *mut u32, payload.len() as u32);
        std::ptr::copy(payload.as_ptr(), result.add(4), payload.len());

        vm.stack_push(Value::String(StrPointer(ptr.as_ptr())))?;
    }

    Ok(())
}

pub fn instr_jump(
    instr_ptr: &mut usize,
    program: &CaoProgram,
    runtime_data: &mut RuntimeData,
) -> ExecutionResult {
    let label: Key = unsafe { decode_value(&program.bytecode, instr_ptr) };
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
                .ok_or(ExecutionError::MissingArgument)?,
        })
        .map_err(|_| ExecutionError::CallStackOverflow)?;

    // set the instr_ptr to the new lane's beginning
    *instr_ptr = program.labels.0.get(label).expect("Label not found").pos as usize;
    Ok(())
}

pub fn execute_call<T>(vm: &mut Vm<T>, instr_ptr: &mut usize, bytecode: &[u8]) -> ExecutionResult {
    let fun_hash: Key = unsafe { decode_value(bytecode, instr_ptr) };
    let procedure = vm
        .callables
        .remove(fun_hash)
        .ok_or(ExecutionError::ProcedureNotFound(fun_hash))?;
    let res = procedure.fun.call(vm);
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
            ExecutionError::VarNotFound(format!("Failed to set local variable: {}", err))
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
            return Err(ExecutionError::BadReturn {
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
            return Err(ExecutionError::BadReturn {
                reason: "Failed to find return address".to_string(),
            });
        }
    }
    // push the return value
    vm.stack_push(value)?;
    Ok(())
}

pub fn begin_repeat<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let n = vm.runtime_data.stack.pop();
    let n = i64::try_from(n).map_err(|_| {
        ExecutionError::invalid_argument("Repeat input must be an integer".to_string())
    })?;
    if n < 0 {
        return Err(ExecutionError::invalid_argument(
            "Repeat input must be non-negative".to_string(),
        ));
    }
    // restore the counter if it's valid, to be used by Repeat
    vm.stack_push(n)?;
    Ok(())
}

/// return i
pub fn repeat<T>(vm: &mut Vm<T>) -> Result<i64, ExecutionError> {
    let i = vm.runtime_data.stack.pop();
    let mut i = i64::try_from(i).map_err(|_| {
        ExecutionError::invalid_argument("Repeat input must be an integer".to_string())
    })?;
    i -= 1;
    if i >= 0 {
        // restore the variable
        vm.stack_push(i)?;
    }

    Ok(i)
}

pub fn instr_copy_last<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let val = vm.runtime_data.stack.last();
    vm.runtime_data
        .stack
        .push(val)
        .map_err(|_| ExecutionError::Stackoverflow)?;

    Ok(())
}

/// push N and I onto the stack
/// I shall be counted down (N-1..0)
pub fn begin_for_each<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let item = vm.runtime_data.stack.last();
    if !item.is_obj() {
        return Err(ExecutionError::invalid_argument(
            "ForEach is only valid for tables".to_string(),
        ));
    }

    let item = vm.get_table(item)?;
    let n = item.len() as i64;

    vm.stack_push(n)?;
    vm.stack_push(0)?;

    Ok(())
}

pub fn for_each<T>(vm: &mut Vm<T>) -> Result<bool, ExecutionError> {
    let i = vm.stack_pop();
    let n = vm.runtime_data.stack.peek_last(0);
    let obj_val = vm.runtime_data.stack.peek_last(1);

    let mut i = i64::try_from(i).expect("Repeat input #0 must be an integer");
    let n = i64::try_from(n).expect("Repeat input #1 must be an integer");
    let obj = vm
        .get_table(obj_val)
        .expect("Repeat input #2 must be a table");

    debug_assert!(0 <= i);

    let should_continue = i < n;
    if should_continue {
        let key = obj.iter().nth(i as usize).map(|(k, _)| k).ok_or_else(|| {
            ExecutionError::invalid_argument(format!(
                "ForEach can not be completed. i: {} n: {}",
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
        vm.stack_pop(); // clean up N
        vm.stack_pop(); // clean up the object
        vm.stack_push(false)?; // for the next GotoIfTrue
    }

    Ok(should_continue)
}
