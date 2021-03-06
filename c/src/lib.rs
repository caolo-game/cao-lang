use std::{alloc, ffi::c_void};

use alloc::Layout;
use cao_lang::{
    compiler::{compile, CaoIr, CompilationError},
    program::CaoProgram,
};

/// Opaque CompiledProgram wrapper.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CompiledProgram {
    _inner: *mut c_void,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum CompileResult {
    cao_CompileResult_Ok,
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
}

#[no_mangle]
pub unsafe extern "C" fn cao_new_compiled_program() -> CompiledProgram {
    CompiledProgram {
        _inner: std::ptr::null_mut(),
    }
}

/// # SAFETY
///
/// Must be called once per CompiledProgram
#[no_mangle]
pub unsafe extern "C" fn cao_free_compiled_program(program: CompiledProgram) {
    if !program._inner.is_null() {
        alloc::dealloc(program._inner as *mut u8, Layout::new::<CaoProgram>());
    }
}

/// Compile a json serialized CaoIR
///
/// # SAFETY
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
    result: *mut CompiledProgram,
) -> CompileResult {
    assert!(!cao_ir.is_null());
    assert!(!result.is_null());

    let cao_ir = std::slice::from_raw_parts(cao_ir, cao_ir_len as usize);

    let ir: CaoIr = match serde_json::from_slice(cao_ir) {
        Ok(ir) => ir,
        Err(_) => return CompileResult::cao_CompileResult_BadJson,
    };

    let program = match compile(ir, None) {
        Ok(p) => p,
        Err(err) => match err {
            CompilationError::Unimplemented(_) => {
                return CompileResult::cao_CompileResult_Unimplmeneted
            }
            CompilationError::EmptyProgram => return CompileResult::cao_CompileResult_EmptyProgram,
            CompilationError::TooManyLanes => return CompileResult::cao_CompileResult_TooManyLanes,
            CompilationError::TooManyCards(_) => {
                return CompileResult::cao_CompileResult_TooManyCards
            }
            CompilationError::DuplicateName(_) => {
                return CompileResult::cao_CompileResult_DuplicateName
            }
            CompilationError::MissingSubProgram(_) => {
                return CompileResult::cao_CompileResult_MissingSubProgram
            }
            CompilationError::MissingNode(_) => {
                return CompileResult::cao_CompileResult_MissingNode
            }
            CompilationError::InvalidJump { .. } => {
                return CompileResult::cao_CompileResult_InvalidJump
            }
            CompilationError::InternalError => {
                return CompileResult::cao_CompileResult_InternalError
            }
            CompilationError::TooManyLocals => {
                return CompileResult::cao_CompileResult_TooManyLocals
            }
            CompilationError::BadVariableName(_) => {
                return CompileResult::cao_CompileResult_BadVariableName
            }
        },
    };
    let program_ptr = alloc::alloc(Layout::new::<CaoProgram>());
    std::ptr::write(program_ptr as *mut CaoProgram, program);

    let program = CompiledProgram {
        _inner: program_ptr as *mut c_void,
    };

    std::ptr::write(result, program);

    CompileResult::cao_CompileResult_Ok
}
