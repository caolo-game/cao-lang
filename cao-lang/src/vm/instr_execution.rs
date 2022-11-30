use std::convert::TryFrom;

use tracing::debug;

use crate::{
    bytecode::{decode_str, read_from_bytes, TriviallyEncodable},
    collections::handle_table::Handle,
    compiled_program::CaoCompiledProgram,
    procedures::ExecutionErrorPayload,
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
    program: &CaoCompiledProgram,
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
                    .get(Handle::from_u32(varid))
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "<<<Unknown variable>>>".to_string()),
            )
        })?;
    runtime_data
        .value_stack
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
    let scalar = runtime_data.value_stack.pop();
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
        Value::Integer(_) | Value::Real(_) => 1,
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
    program: &CaoCompiledProgram,
) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let payload = read_str(&mut (handle as usize), program.data.as_slice())
        .ok_or(ExecutionErrorPayload::InvalidArgument { context: None })?;

    let ptr = vm.init_string(payload)?;
    vm.stack_push(Value::String(ptr))?;

    Ok(())
}

pub fn instr_jump(
    src_ptr: usize,
    instr_ptr: &mut usize,
    program: &CaoCompiledProgram,
    runtime_data: &mut RuntimeData,
) -> ExecutionResult {
    let label: Handle = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let argcount: u32 = unsafe { decode_value(&program.bytecode, instr_ptr) };

    // remember the location after this jump
    runtime_data
        .call_stack
        .last_mut()
        .expect("Call stack was empty")
        .dst_instr_ptr = *instr_ptr as u32;

    // init the new call frame
    runtime_data
        .call_stack
        .push(CallFrame {
            src_instr_ptr: src_ptr as u32,
            dst_instr_ptr: *instr_ptr as u32,
            stack_offset: runtime_data
                .value_stack
                .len()
                .checked_sub(argcount as usize)
                .ok_or(ExecutionErrorPayload::MissingArgument)? as u32,
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

fn write_local_var<T>(vm: &mut Vm<T>, handle: u32, value: Value, offset: usize) -> ExecutionResult {
    vm.runtime_data
        .value_stack
        .set(offset + handle as usize, value)
        .map_err(|err| {
            ExecutionErrorPayload::VarNotFound(format!("Failed to set local variable: {}", err))
        })?;
    Ok(())
}

fn stack_offset<T>(vm: &Vm<T>) -> usize {
    let offset = vm
        .runtime_data
        .call_stack
        .last()
        .expect("Call stack is emtpy")
        .stack_offset;
    offset as usize
}

pub fn set_local<T>(vm: &mut Vm<T>, bytecode: &[u8], instr_ptr: &mut usize) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let offset = stack_offset(vm);
    let value = vm.runtime_data.value_stack.pop_w_offset(offset);
    write_local_var(vm, handle, value, offset)
}

fn read_local_var<T>(vm: &mut Vm<T>, handle: u32) -> Result<Value, ExecutionErrorPayload> {
    let offset = stack_offset(vm);
    let value = vm.runtime_data.value_stack.get(offset + handle as usize);
    Ok(value)
}

pub fn get_local<T>(vm: &mut Vm<T>, bytecode: &[u8], instr_ptr: &mut usize) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let value = read_local_var(vm, handle)?;
    vm.stack_push(value)?;
    Ok(())
}

pub fn instr_return<T>(vm: &mut Vm<T>, instr_ptr: &mut usize) -> ExecutionResult {
    // pop the current stack frame
    let value = match vm.runtime_data.call_stack.pop() {
        // return value
        Some(rt) => vm
            .runtime_data
            .value_stack
            .clear_until(rt.stack_offset as usize),
        None => {
            return Err(ExecutionErrorPayload::BadReturn {
                reason: "Call stack is empty".to_string(),
            })
        }
    };
    // read the previous frame
    match vm.runtime_data.call_stack.last_mut() {
        Some(CallFrame {
            dst_instr_ptr: ptr, ..
        }) => {
            *instr_ptr = *ptr as usize;
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

pub fn instr_copy_last<T>(vm: &mut Vm<T>) -> ExecutionResult {
    let val = vm.runtime_data.value_stack.last();
    vm.runtime_data
        .value_stack
        .push(val)
        .map_err(|_| ExecutionErrorPayload::Stackoverflow)?;

    Ok(())
}

/// push `i=0` onto the stack
pub fn begin_for_each<T>(
    vm: &mut Vm<T>,
    bytecode: &[u8],
    instr_ptr: &mut usize,
) -> ExecutionResult {
    let i_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let t_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let item_val = vm.runtime_data.value_stack.last();
    // test if the input is a table
    let item = vm.get_table_mut(item_val)?;
    debug!("Starting for-each on table {:?}", item);
    item.rebuild_keys();
    let offset = stack_offset(vm);
    write_local_var(vm, i_handle, Value::Integer(0), offset)?;
    write_local_var(vm, t_handle, item_val, offset)?;

    Ok(())
}

/// Assumes that [begin_for_each](begin_for_each) was called once to set up the loop
///
/// Pushes the next key and the object onto the stack. Assumes that the lane takes these as
/// parameters.
///
/// Pushes should_continue on top of the stack
pub fn for_each<T>(vm: &mut Vm<T>, bytecode: &[u8], instr_ptr: &mut usize) -> ExecutionResult {
    let loop_variable: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let t_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };

    let i_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let k_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };
    let v_handle: u32 = unsafe { decode_value(bytecode, instr_ptr) };

    let offset = stack_offset(vm);
    let i = read_local_var(vm, loop_variable)?;
    let obj_val = read_local_var(vm, t_handle)?;

    let mut i = i64::try_from(i).map_err(|_| {
        ExecutionErrorPayload::AssertionError("ForEach i must be an integer. This error can be caused by stack corruption. Check your function calls!".to_string())
    })?;
    let obj = vm.get_table(obj_val).map_err(|_| {
        ExecutionErrorPayload::AssertionError("ForEach value is not an object. This error can be caused by stack corruption. Check your function calls!".to_string())
    })?;

    debug_assert!(0 <= i, "for_each overflow");

    let n = obj.len() as i64;

    let should_continue = 0 <= i && i < n;
    if should_continue {
        let key = obj.nth_key(i as usize);
        let val = obj.get(&key).copied().unwrap_or(Value::Nil);
        i += 1;

        write_local_var(vm, v_handle, val, offset)?;
        write_local_var(vm, k_handle, key, offset)?;
        write_local_var(vm, i_handle, Value::Integer(i), offset)?;
        // store the loop variable
        write_local_var(vm, loop_variable, Value::Integer(i), offset)?;
    }
    vm.stack_push(should_continue)?;

    Ok(())
}
