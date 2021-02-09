use std::{convert::TryFrom, mem};

use slog::{debug, o, trace, warn, Logger};

use crate::{
    collections::pre_hash_map::Key, instruction::Instruction, procedures::ExecutionError,
    procedures::ExecutionResult, program::CompiledProgram, scalar::Scalar,
    traits::ByteDecodeProperties, traits::Callable, traits::DecodeInPlace, traits::MAX_STR_LEN,
    Pointer, VariableId,
};

use super::{data::RuntimeData, HistoryEntry, Vm};

#[inline]
pub fn read_str<'a>(bytecode_pos: &mut usize, program: &'a [u8]) -> Option<&'a str> {
    let p = *bytecode_pos;
    let limit = program.len().min(p + MAX_STR_LEN);
    let (len, s): (_, &'a str) =
        <&'a str as DecodeInPlace>::decode_in_place(&program[p..limit]).ok()?;
    *bytecode_pos += len;
    Some(s)
}

/// # Safety
///
/// Assumes that the underlying data is safely decodable to the given type
///
#[inline]
pub unsafe fn decode_value<T: ByteDecodeProperties>(
    logger: &Logger,
    bytes: &[u8],
    bytecode_pos: &mut usize,
) -> T {
    trace!(
        logger,
        "Decoding value of type {} at bytecode_pos {}, len: {}",
        std::any::type_name::<T>(),
        bytecode_pos,
        bytes.len()
    );
    let (len, val) = T::decode_unsafe(&bytes[*bytecode_pos..]);
    *bytecode_pos += len;
    val
}

#[allow(unused)]
#[inline]
pub fn decode_in_place<'a, T: DecodeInPlace<'a>>(
    logger: &Logger,
    bytes: &'a [u8],
    bytecode_pos: &mut usize,
) -> Result<T::Ref, ExecutionError> {
    trace!(
        logger,
        "Decoding value of type {} at bytecode_pos {}, len: {}",
        std::any::type_name::<T>(),
        bytecode_pos,
        bytes.len()
    );
    let (len, val) = T::decode_in_place(&bytes[*bytecode_pos..])
        .map_err(|_| ExecutionError::invalid_argument("Failed to decode value".to_owned()))?;
    *bytecode_pos += len;
    trace!(
        logger,
        "Decoding successful, new bytecode_pos {}",
        bytecode_pos,
    );
    Ok(val)
}

#[inline]
pub fn instr_read_var<'a>(
    logger: &Logger,
    runtime_data: &mut RuntimeData,
    bytecode: &'a [u8],
    bytecode_pos: &mut usize,
) -> ExecutionResult {
    let VariableId(varname) = unsafe { decode_value(&logger, bytecode, bytecode_pos) };
    let value = runtime_data
        .registers
        .get(varname as usize)
        .ok_or_else(|| {
            debug!(logger, "Variable {} does not exist", varname);
            ExecutionError::invalid_argument(None)
        })?;
    runtime_data
        .stack
        .push(*value)
        .map_err(|_| ExecutionError::Stackoverflow)?;
    Ok(())
}

#[inline]
pub fn instr_set_var(
    logger: &Logger,
    runtime_data: &mut RuntimeData,
    bytecode: &[u8],
    bytecode_pos: &mut usize,
) -> ExecutionResult {
    let varname = unsafe { decode_value::<VariableId>(logger, bytecode, bytecode_pos) };
    let scalar = runtime_data.stack.pop();
    let varname = varname.0 as usize;
    if runtime_data.registers.len() <= varname {
        runtime_data.registers.resize(varname + 1, Scalar::Null);
    }
    runtime_data.registers[varname] = scalar;
    Ok(())
}

#[inline]
pub fn instr_exit(logger: &Logger, runtime_data: &mut RuntimeData) -> Result<i32, ExecutionError> {
    debug!(logger, "Exit called");
    let code = runtime_data.stack.pop();
    if let Scalar::Integer(code) = code {
        debug!(logger, "Exit code {:?}", code);
        return Ok(code);
    }
    Ok(0)
}

#[inline]
pub fn instr_breadcrumb(
    logger: &Logger,
    history: &mut Vec<HistoryEntry>,
    bytecode: &[u8],
    bytecode_pos: &mut usize,
) {
    let nodeid = unsafe { decode_value(logger, &bytecode, bytecode_pos) };

    let instr = bytecode[*bytecode_pos];
    let instr = Instruction::try_from(instr).ok();
    *bytecode_pos += 1;
    trace!(logger, "Logging visited node {:?}", nodeid);
    history.push(HistoryEntry { id: nodeid, instr });
}

#[inline]
pub fn instr_scalar_array(
    logger: &Logger,
    runtime_data: &mut RuntimeData,
    bytecode: &[u8],
    bytecode_pos: &mut usize,
) -> ExecutionResult {
    let len: i32 = unsafe { decode_value(logger, bytecode, bytecode_pos) };
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
    let bytecode_pos = runtime_data.memory.len();
    for _ in 0..len {
        let val = runtime_data.stack.pop();
        runtime_data.write_to_memory(val)?;
    }
    runtime_data
        .stack
        .push(Scalar::Pointer(Pointer(bytecode_pos as u32)))
        .map_err(|_| ExecutionError::Stackoverflow)?;

    Ok(())
}

#[inline]
pub fn instr_string_literal<T>(
    vm: &mut Vm<T>,
    bytecode_pos: &mut usize,
    bytecode: &[u8],
) -> ExecutionResult {
    let literal =
        read_str(bytecode_pos, bytecode).ok_or_else(|| ExecutionError::invalid_argument(None))?;
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

#[inline]
pub fn instr_jump(
    logger: &Logger,
    bytecode_pos: &mut usize,
    program: &CompiledProgram,
) -> ExecutionResult {
    let label: Key = unsafe { decode_value(logger, &program.bytecode, bytecode_pos) };
    *bytecode_pos = program
        .labels
        .0
        .get(label)
        .ok_or(ExecutionError::InvalidLabel(label))?
        .pos as usize;
    Ok(())
}

#[inline]
pub fn jump_if<F: Fn(Scalar) -> bool>(
    logger: &Logger,
    runtime_data: &mut RuntimeData,
    bytecode_pos: &mut usize,
    program: &CompiledProgram,
    predicate: F,
) -> Result<(), ExecutionError> {
    if runtime_data.stack.is_empty() {
        warn!(
            logger,
            "JumpIfTrue called with missing arguments, stack: {:?}", runtime_data.stack
        );
        return Err(ExecutionError::invalid_argument(None));
    }
    let cond = runtime_data.stack.pop();
    let label: Key = unsafe { decode_value(logger, &program.bytecode, bytecode_pos) };
    if predicate(cond) {
        *bytecode_pos = program
            .labels
            .0
            .get(label)
            .ok_or(ExecutionError::InvalidLabel(label))?
            .pos as usize;
    }
    Ok(())
}

#[inline]
pub fn execute_call<T>(
    vm: &mut Vm<T>,
    bytecode_pos: &mut usize,
    bytecode: &[u8],
) -> Result<(), ExecutionError> {
    let fun_hash = unsafe { decode_value(&vm.logger, bytecode, bytecode_pos) };
    let mut fun = vm
        .callables
        .remove(fun_hash)
        .ok_or(ExecutionError::ProcedureNotFound(fun_hash))?;
    let logger = vm.logger.new(o!("function"=> fun.name.to_string()));
    let res = (|| {
        let n_inputs = fun.num_params();
        let mut inputs = Vec::with_capacity(n_inputs as usize);
        for _ in 0..n_inputs {
            let arg = vm.runtime_data.stack.pop();
            inputs.push(arg)
        }
        debug!(logger, "Calling function with inputs: {:?}", inputs);
        fun.call(vm, &inputs).map_err(|e| {
            warn!(logger, "Calling function failed with {:?}", e);
            e
        })?;
        debug!(logger, "Function call returned");

        Ok(())
    })();
    // cleanup
    vm.callables.insert(fun_hash, fun);
    res
}
