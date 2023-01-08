use std::sync::Arc;

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    wrap_pyfunction,
};

#[pyclass]
#[derive(Clone)]
pub struct CompilationUnit {
    // wrap in an Arc, because clone might cause lifetime issues
    inner: Arc<cao_lang::prelude::CaoProgram>,
}

#[pyclass]
#[derive(Clone)]
pub struct CompilationOptions {
    inner: cao_lang::prelude::CompileOptions,
}

#[pymethods]
impl CompilationUnit {
    #[staticmethod]
    fn from_json(payload: &str) -> PyResult<Self> {
        let inner =
            serde_json::from_str(payload).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[staticmethod]
    fn from_yaml(payload: &str) -> PyResult<Self> {
        let inner =
            serde_yaml::from_str(payload).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

#[pymethods]
impl CompilationOptions {
    #[new]
    fn new() -> Self {
        let inner = cao_lang::prelude::CompileOptions::default();
        Self { inner }
    }

    #[getter(recursion_limit)]
    pub fn get_recursion_limit(&self) -> u32 {
        self.inner.recursion_limit
    }

    #[setter(recursion_limit)]
    pub fn set_recursion_limit(&mut self, value: u32) {
        self.inner.recursion_limit = value;
    }
}

#[pyclass]
#[derive(Clone)]
pub struct CaoCompiledProgram {
    inner: Arc<cao_lang::prelude::CaoCompiledProgram>,
}

#[pyfunction]
fn compile(
    cu: CompilationUnit,
    options: Option<CompilationOptions>,
) -> PyResult<CaoCompiledProgram> {
    cao_lang::prelude::compile(cu.inner.as_ref().clone(), options.map(|o| o.inner))
        .map_err(|err| PyValueError::new_err(err.to_string()))
        .map(|inner| CaoCompiledProgram {
            inner: Arc::new(inner),
        })
}

#[pyfunction]
fn run(prog: CaoCompiledProgram) -> PyResult<()> {
    let mut vm = cao_lang::prelude::Vm::new(()).expect("Failed to init vm");
    vm.run(&prog.inner)
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))
        .map(|_| ())
}

/// Return the version of the native Cao-Lang used
#[pyfunction]
fn native_version() -> PyResult<String> {
    Ok(cao_lang::version::VERSION_STR.to_string())
}

#[pymodule]
fn cao_lang_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(native_version, m)?)?;

    m.add_class::<CompilationUnit>()?;
    m.add_class::<CompilationOptions>()?;
    m.add_class::<CaoCompiledProgram>()?;

    Ok(())
}
