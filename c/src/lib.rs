use std::{alloc, ffi::c_void};

use alloc::Layout;
use cao_lang::{
    compiler::{compile, CaoIr, CompilationErrorPayload},
    program::CaoProgram,
    vm::Vm,
};

/// Opaque CompiledProgram wrapper.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CaoCompiledProgram {
    _inner: *mut c_void,
}

/// Opaque VM wrapper.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CaoVm {
    _inner: *mut c_void,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum CompileResult {
    cao_CompileResult_Ok = 0,
    cao_CompileResult_BadJson,
    cao_CompileResult_Unimplmeneted,
    cao_CompileResult_EmptyProgram,
    cao_CompileResult_TooManyLanes,
    cao_CompileResult_TooManyCards,
    cao_CompileResult_DuplicateName,
    cao_CompileResult_MissingSubProgram,
    cao_CompileResult_MissingNode,
    cao_CompileResult_InvalidJump,
    cao_CompileResult_InternalError,
    cao_CompileResult_TooManyLocals,
    cao_CompileResult_BadVariableName,
    cao_CompileResult_EmptyVariable,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum ExecutionResult {
    cao_ExecutionResult_Ok = 0,
    /// VM was not initialized
    cao_ExecutionResult_BadVm,
    /// Program was not initialized
    cao_ExecutionResult_BadProgram,
    cao_ExecutionResult_CallStackOverflow,
    cao_ExecutionResult_UnexpectedEndOfInput,
    cao_ExecutionResult_ExitCode,
    cao_ExecutionResult_InvalidInstruction,
    cao_ExecutionResult_InvalidArgument,
    cao_ExecutionResult_VarNotFound,
    cao_ExecutionResult_ProcedureNotFound,
    cao_ExecutionResult_Unimplemented,
    cao_ExecutionResult_OutOfMemory,
    cao_ExecutionResult_MissingArgument,
    cao_ExecutionResult_Timeout,
    cao_ExecutionResult_TaskFailure,
    cao_ExecutionResult_Stackoverflow,
    cao_ExecutionResult_BadReturn,
    cao_ExecutionResult_Unhashable,
}

/// # Safety
///
/// The produced program must be freed by calling
/// [cao_free_compiled_program](cao_free_compiled_program)
#[no_mangle]
pub unsafe extern "C" fn cao_new_compiled_program() -> CaoCompiledProgram {
    CaoCompiledProgram {
        _inner: std::ptr::null_mut(),
    }
}

/// # Safety
///
/// This function will panic if initial memory allocation fails
///
/// The produced VM must be freed by calling
/// [cao_free_vm](cao_free_vm)
#[no_mangle]
pub unsafe extern "C" fn cao_new_vm() -> CaoVm {
    let vm = Box::new(
        Vm::<*mut c_void>::new(std::ptr::null_mut()).expect("Failed to initialize the VM"),
    );
    let vm = Box::leak(vm);
    CaoVm {
        _inner: vm as *mut Vm<*mut c_void> as *mut c_void,
    }
}

/// # Safety
///
/// Must be called once per CaoVm
#[no_mangle]
pub unsafe extern "C" fn cao_free_vm(vm: *mut CaoVm) {
    if vm.is_null() {
        return;
    }
    let vm = &mut *vm;
    if !vm._inner.is_null() {
        let _vm = Box::from_raw(vm._inner as *mut Vm<*mut c_void>);
    }
    vm._inner = std::ptr::null_mut();
}

/// # Safety
///
/// Must be called once per CompiledProgram
#[no_mangle]
pub unsafe extern "C" fn cao_free_compiled_program(program: *mut CaoCompiledProgram) {
    if program.is_null() {
        return;
    }
    let program = &mut *program;
    if !program._inner.is_null() {
        let _program = Box::from_raw(program._inner as *mut CaoProgram);
    }
    program._inner = std::ptr::null_mut();
}

/// Compile a json serialized CaoIR
///
/// # Safety
///
/// `cao_ir_len` must be the length of the `cao_ir` string
///
/// `cao_ir` must be a valid json serialized CaoIR
///
/// The caller is responsible for freeing the produced program.
///
#[no_mangle]
pub unsafe extern "C" fn cao_compile_json(
    cao_ir: *const u8,
    cao_ir_len: u32,
    result: *mut CaoCompiledProgram,
) -> CompileResult {
    assert!(!cao_ir.is_null());
    assert!(!result.is_null());

    let cao_ir = std::slice::from_raw_parts(cao_ir, cao_ir_len as usize);

    let ir: CaoIr = match serde_json::from_slice(cao_ir) {
        Ok(ir) => ir,
        Err(_) => return CompileResult::cao_CompileResult_BadJson,
    };

    let program = match compile(&ir, None) {
        Ok(p) => p,
        Err(err) => match err.payload {
            CompilationErrorPayload::Unimplemented(_) => {
                return CompileResult::cao_CompileResult_Unimplmeneted
            }
            CompilationErrorPayload::EmptyProgram => {
                return CompileResult::cao_CompileResult_EmptyProgram
            }
            CompilationErrorPayload::TooManyLanes => {
                return CompileResult::cao_CompileResult_TooManyLanes
            }
            CompilationErrorPayload::TooManyCards(_) => {
                return CompileResult::cao_CompileResult_TooManyCards
            }
            CompilationErrorPayload::DuplicateName(_) => {
                return CompileResult::cao_CompileResult_DuplicateName
            }
            CompilationErrorPayload::MissingSubProgram(_) => {
                return CompileResult::cao_CompileResult_MissingSubProgram
            }
            CompilationErrorPayload::MissingNode(_) => {
                return CompileResult::cao_CompileResult_MissingNode
            }
            CompilationErrorPayload::InvalidJump { .. } => {
                return CompileResult::cao_CompileResult_InvalidJump
            }
            CompilationErrorPayload::InternalError => {
                return CompileResult::cao_CompileResult_InternalError
            }
            CompilationErrorPayload::TooManyLocals => {
                return CompileResult::cao_CompileResult_TooManyLocals
            }
            CompilationErrorPayload::BadVariableName(_) => {
                return CompileResult::cao_CompileResult_BadVariableName
            }
            CompilationErrorPayload::EmptyVariable => {
                return CompileResult::cao_CompileResult_EmptyVariable
            }
        },
    };
    let program_ptr = alloc::alloc(Layout::new::<CaoProgram>());
    std::ptr::write(program_ptr as *mut CaoProgram, program);

    let program = CaoCompiledProgram {
        _inner: program_ptr as *mut c_void,
    };

    std::ptr::write(result, program);

    CompileResult::cao_CompileResult_Ok
}

/// # Safety
///
/// Runs a previously compiled program in the given VM
#[no_mangle]
pub unsafe extern "C" fn cao_run_program(
    program: CaoCompiledProgram,
    vm: CaoVm,
) -> ExecutionResult {
    if program._inner.is_null() {
        return ExecutionResult::cao_ExecutionResult_BadProgram;
    }
    if vm._inner.is_null() {
        return ExecutionResult::cao_ExecutionResult_BadVm;
    }
    let program: &CaoProgram = &*(program._inner as *const _);
    let vm: &mut Vm<*mut c_void> = &mut *(vm._inner as *mut _);

    match vm.run(program) {
        Ok(_) => {}
        Err(err) => match err {
            cao_lang::procedures::ExecutionError::CallStackOverflow => {
                return ExecutionResult::cao_ExecutionResult_CallStackOverflow
            }
            cao_lang::procedures::ExecutionError::UnexpectedEndOfInput => {
                return ExecutionResult::cao_ExecutionResult_UnexpectedEndOfInput
            }
            cao_lang::procedures::ExecutionError::ExitCode(_) => {
                return ExecutionResult::cao_ExecutionResult_ExitCode
            }
            cao_lang::procedures::ExecutionError::InvalidInstruction(_) => {
                return ExecutionResult::cao_ExecutionResult_InvalidInstruction
            }
            cao_lang::procedures::ExecutionError::InvalidArgument { .. } => {
                return ExecutionResult::cao_ExecutionResult_InvalidArgument
            }
            cao_lang::procedures::ExecutionError::VarNotFound(_) => {
                return ExecutionResult::cao_ExecutionResult_VarNotFound
            }
            cao_lang::procedures::ExecutionError::ProcedureNotFound(_) => {
                return ExecutionResult::cao_ExecutionResult_ProcedureNotFound
            }
            cao_lang::procedures::ExecutionError::Unimplemented => {
                return ExecutionResult::cao_ExecutionResult_Unimplemented
            }
            cao_lang::procedures::ExecutionError::OutOfMemory => {
                return ExecutionResult::cao_ExecutionResult_OutOfMemory
            }
            cao_lang::procedures::ExecutionError::MissingArgument => {
                return ExecutionResult::cao_ExecutionResult_MissingArgument
            }
            cao_lang::procedures::ExecutionError::Timeout => {
                return ExecutionResult::cao_ExecutionResult_Timeout
            }
            cao_lang::procedures::ExecutionError::TaskFailure(_) => {
                return ExecutionResult::cao_ExecutionResult_TaskFailure
            }
            cao_lang::procedures::ExecutionError::Stackoverflow => {
                return ExecutionResult::cao_ExecutionResult_Stackoverflow
            }
            cao_lang::procedures::ExecutionError::BadReturn { .. } => {
                return ExecutionResult::cao_ExecutionResult_BadReturn
            }
            cao_lang::procedures::ExecutionError::Unhashable => {
                return ExecutionResult::cao_ExecutionResult_Unhashable
            }
        },
    }

    ExecutionResult::cao_ExecutionResult_Ok
}
