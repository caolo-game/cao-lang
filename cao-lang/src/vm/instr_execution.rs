use std::{convert::TryFrom, mem};

use crate::{
    collections::pre_hash_map::Key, instruction::Instruction, procedures::ExecutionError,
    procedures::ExecutionResult, program::CaoProgram, scalar::Scalar, traits::ByteDecodeProperties,
    traits::DecodeInPlace, traits::MAX_STR_LEN, Pointer, VariableId,
};

use super::{
    data::{CallFrame, RuntimeData},
    HistoryEntry, Vm,
};

pub fn read_str<'a>(instr_ptr: &mut usize, program: &'a [u8]) -> Option<&'a str> {
    let p = *instr_ptr;
    let limit = program.len().min(p + MAX_STR_LEN);
    let (len, s): (_, &'a str) =
        <&'a str as DecodeInPlace>::decode_in_place(&program[p..limit]).ok()?;
    *instr_ptr += len;
    Some(s)
}

/// # Safety
///
/// Assumes that the underlying data is safely decodable to the given type
///
pub unsafe fn decode_value<T: ByteDecodeProperties>(bytes: &[u8], instr_ptr: &mut usize) -> T {
    let (len, val) = T::decode_unsafe(&bytes[*instr_ptr..]);
    *instr_ptr += len;
    val
}

#[allow(unused)]
pub fn decode_in_place<'a, T: DecodeInPlace<'a>>(
    bytes: &'a [u8],
    instr_ptr: &mut usize,
) -> Result<T::Ref, ExecutionError> {
    let (len, val) = T::decode_in_place(&bytes[*instr_ptr..])
        .map_err(|_| ExecutionError::invalid_argument("Failed to decode value".to_owned()))?;
    *instr_ptr += len;
    Ok(val)
}

pub fn instr_read_var<'a>(
    runtime_data: &mut RuntimeData,
    bytecode: &'a [u8],
    instr_ptr: &mut usize,
) -> ExecutionResult {
    let VariableId(varid) = unsafe { decode_value(bytecode, instr_ptr) };
    let value = runtime_data
        .global_vars
        .get(varid as usize)
        .ok_or_else(|| {
            ExecutionError::invalid_argument(format!("Variable {} does not exist", varid))
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
        runtime_data.global_vars.resize(varid + 1, Scalar::Null);
    }
    runtime_data.global_vars[varid] = scalar;
    Ok(())
}

pub fn instr_exit(runtime_data: &mut RuntimeData) -> Result<i32, ExecutionError> {
    let code = runtime_data.stack.pop();
    if let Scalar::Integer(code) = code {
        return Ok(code);
    }
    Ok(0)
}

pub fn instr_breadcrumb(history: &mut Vec<HistoryEntry>, bytecode: &[u8], instr_ptr: &mut usize) {
    let nodeid = unsafe { decode_value(&bytecode, instr_ptr) };

    let instr = bytecode[*instr_ptr];
    let instr = Instruction::try_from(instr).ok();
    *instr_ptr += 1;
    history.push(HistoryEntry { id: nodeid, instr });
}

pub fn instr_scalar_array(
    runtime_data: &mut RuntimeData,
    bytecode: &[u8],
    instr_ptr: &mut usize,
) -> ExecutionResult {
    let len: i32 = unsafe { decode_value(bytecode, instr_ptr) };
    if len < 0 {
        return Err(ExecutionError::invalid_argument(
            "ScalarArray length must be positive integer".to_owned(),
        ));
    }
    if len as usize > runtime_data.stack.len() {
        return Err(ExecutionError::invalid_argument(format!(
            "The stack holds {} items, but ScalarArray requested {}",
            runtime_data.stack.len(),
            len,
        )));
    }
    let instr_ptr = runtime_data.memory.len();
    for _ in 0..len {
        let val = runtime_data.stack.pop();
        runtime_data.write_to_memory(val)?;
    }
    runtime_data
        .stack
        .push(Scalar::Pointer(Pointer(instr_ptr as u32)))
        .map_err(|_| ExecutionError::Stackoverflow)?;

    Ok(())
}

pub fn instr_string_literal<T>(
    vm: &mut Vm<T>,
    instr_ptr: &mut usize,
    program: &CaoProgram,
) -> ExecutionResult {
    let handle: u32 = unsafe { decode_value(&program.bytecode, instr_ptr) };
    let literal = read_str(&mut (handle as usize), program.data.as_slice())
        .ok_or_else(|| ExecutionError::invalid_argument(None))?;
    let obj = vm.set_value_with_decoder(literal, |o, vm| {
        // SAFETY
        // As long as the same Vm instance's accessors are used this should be
        // fine (tm)
        let res = vm.get_value_in_place::<&str>(o.index.unwrap()).unwrap();
        let res: &'static str = unsafe { mem::transmute(res) };
        Box::new(res)
    })?;
    vm.runtime_data
        .stack
        .push(Scalar::Pointer(obj.index.unwrap()))
        .map_err(|_| ExecutionError::Stackoverflow)?;
    Ok(())
}

pub fn instr_jump(
    instr_ptr: &mut usize,
    program: &CaoProgram,
    runtime_data: &mut RuntimeData,
) -> ExecutionResult {
    let label: Key = unsafe { decode_value(&program.bytecode, instr_ptr) };

    // remember the location after this jump
    runtime_data
        .call_stack
        .last_mut()
        .ok_or_else(|| ExecutionError::CallStackEmpty)?
        .instr_ptr = *instr_ptr;

    // init the new call frame
    runtime_data
        .call_stack
        .push(CallFrame {
            instr_ptr: *instr_ptr,
            stack_offset: runtime_data.stack.len(),
        })
        .map_err(|_| ExecutionError::CallStackOverflow)?;

    // set the instr_ptr to the new lane's beginning
    *instr_ptr = program
        .labels
        .0
        .get(label)
        .ok_or(ExecutionError::InvalidLabel(label))?
        .pos as usize;
    Ok(())
}

pub fn execute_call<T>(
    vm: &mut Vm<T>,
    instr_ptr: &mut usize,
    bytecode: &[u8],
) -> Result<(), ExecutionError> {
    let fun_hash = unsafe { decode_value(bytecode, instr_ptr) };
    let fun = vm
        .callables
        .remove(fun_hash)
        .ok_or(ExecutionError::ProcedureNotFound(fun_hash))?;
    let res = fun.fun.call(vm);
    //cleanup
    vm.callables.insert(fun_hash, fun);
    res
}
