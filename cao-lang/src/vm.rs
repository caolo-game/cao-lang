use crate::compiler::NodeId;
use crate::instruction::Instruction;
use crate::prelude::*;
use crate::scalar::Scalar;
use crate::VarName;
use crate::{binary_compare, pop_stack};
use serde::{Deserialize, Serialize};
use slog::{debug, trace, warn};
use slog::{o, Drain, Logger};
use std::collections::HashMap;
use std::convert::TryFrom;

type ConvertFn<Aux> = unsafe fn(&Object, &VM<Aux>) -> Box<dyn ObjectProperties>;

#[derive(Debug, Clone, Copy)]
pub enum ConvertError {
    /// Null object was passed to convert
    NullPtr,
    BadType,
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    /// nullable index of the Object's data in the VM memory
    pub index: Option<TPointer>,
    /// size of the data in the VM memory
    pub size: u32,
}

impl Default for Object {
    fn default() -> Self {
        Self::null()
    }
}

impl Object {
    pub fn null() -> Self {
        Self {
            index: None,
            size: 0,
        }
    }

    pub fn as_inner<'a, Aux>(
        &self,
        vm: &'a VM<Aux>,
    ) -> Result<Box<dyn ObjectProperties>, ConvertError> {
        self.index
            .ok_or_else(|| ConvertError::NullPtr)
            .map(|index| unsafe { vm.converters[&index](self, vm) })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: NodeId,
    pub instr: Instruction,
}

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary data structure passed to custom functions.
pub struct VM<Aux = ()> {
    pub logger: Logger,
    pub history: Vec<HistoryEntry>,
    pub auxiliary_data: Aux,
    pub max_iter: i32,
    pub memory_limit: usize,

    memory: Vec<u8>,
    stack: Vec<Scalar>,
    callables: HashMap<String, Procedure<Aux>>,
    objects: HashMap<TPointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<TPointer, ConvertFn<Aux>>,
    variables: HashMap<VarName, Scalar>,
}

impl<Aux> VM<Aux> {
    pub fn new(logger: impl Into<Option<Logger>>, auxiliary_data: Aux) -> Self {
        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        Self {
            logger,
            history: Vec::new(),
            converters: HashMap::new(),
            auxiliary_data,
            memory: Vec::with_capacity(512),
            callables: HashMap::new(),
            memory_limit: 40000,
            stack: Vec::with_capacity(128),
            objects: HashMap::with_capacity(128),
            variables: HashMap::with_capacity(128),
            max_iter: 1000,
        }
    }

    pub fn clear(&mut self) {
        self.memory.clear();
        self.stack.clear();
        self.objects.clear();
        self.converters.clear();
        self.variables.clear();
    }

    pub fn read_var(&self, name: &str) -> Option<Scalar> {
        self.variables.get(name).cloned()
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_iter = max_iter;
        self
    }

    pub fn stack(&self) -> &[Scalar] {
        &self.stack
    }

    pub fn get_aux(&self) -> &Aux {
        &self.auxiliary_data
    }

    pub fn get_aux_mut(&mut self) -> &mut Aux {
        &mut self.auxiliary_data
    }

    pub fn unwrap_aux(self) -> Aux {
        self.auxiliary_data
    }

    pub fn register_function<C: Callable<Aux> + 'static>(&mut self, name: &str, f: C) {
        self.callables.insert(name.to_owned(), Procedure::new(f));
    }

    pub fn register_function_obj(&mut self, name: &str, f: Procedure<Aux>) {
        self.callables.insert(name.to_owned(), f);
    }

    pub fn get_value_in_place<'a, T: DecodeInPlace<'a>>(
        &'a self,
        ptr: TPointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        use std::any::type_name;

        let size = T::BYTELEN;
        let object = self.objects.get(&ptr)?;
        if object.size as usize != size {
            debug!(
                self.logger,
                "Attempting to reference an object with the wrong type ({}) at address {}",
                type_name::<T>(),
                ptr
            );
            return None;
        }
        match object.index {
            Some(index) => {
                let data = &self.memory;
                let head = index as usize;
                let tail = (head.checked_add(size as usize))
                    .unwrap_or(data.len())
                    .min(data.len());
                T::decode_in_place(&data[head..tail]).ok()
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }
    pub fn get_value<T: ByteEncodeProperties>(&self, ptr: TPointer) -> Option<T> {
        let size = T::BYTELEN;
        let object = self.objects.get(&ptr)?;
        if object.size as usize != size {
            debug!(
                self.logger,
                "Attempting to reference an object with the wrong type ({}) at address {}",
                T::displayname(),
                ptr
            );
            return None;
        }
        match object.index {
            Some(index) => {
                let data = &self.memory;
                let head = index as usize;
                let tail = (head + size).min(data.len());
                T::decode(&data[head..tail]).ok()
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    fn write_to_memory<T: ByteEncodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<TPointer, ExecutionError> {
        let result = self.memory.len();
        let bytes = val.encode().map_err(|err| {
            warn!(self.logger, "Failed to encode argument {:?}", err);
            ExecutionError::invalid_argument(None)
        })?;

        // second part defends against integer overflow attacks
        if bytes.len() + result >= self.memory_limit || bytes.len() >= self.memory_limit {
            return Err(ExecutionError::OutOfMemory);
        }

        self.memory.extend(bytes.iter());
        Ok(result as TPointer)
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value<T: ByteEncodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<Object, ExecutionError> {
        let result = self.write_to_memory(val)?;
        let object = Object {
            index: Some(result as i32),
            size: T::BYTELEN as u32,
        };
        self.objects.insert(result, object);
        self.converters
            .insert(result as TPointer, |o: &Object, vm: &VM<Aux>| {
                let res: T = vm.get_value(o.index.unwrap()).unwrap();
                Box::new(res)
            });

        self.stack_push(Scalar::Pointer(result as TPointer))?;

        debug!(
            self.logger,
            "Set value {:?} {:?} {}",
            object,
            T::BYTELEN,
            T::displayname()
        );

        Ok(object)
    }

    pub fn stack_push<S>(&mut self, value: S) -> Result<(), ExecutionError>
    where
        S: Into<Scalar>,
    {
        self.stack.push(value.into());
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Option<Scalar> {
        self.stack.pop()
    }

    #[inline]
    fn decode_value<T: ByteEncodeProperties>(
        logger: &Logger,
        bytes: &[u8],
        ptr: &mut usize,
    ) -> Result<T, ExecutionError> {
        trace!(
            logger,
            "Decoding value at ptr {}, len: {}",
            ptr,
            bytes.len()
        );
        let len = T::BYTELEN;
        if *ptr + len > bytes.len() {
            return Err(ExecutionError::UnexpectedEndOfInput);
        }
        let val = T::decode(&bytes[*ptr..*ptr + len])
            .map_err(|_| ExecutionError::invalid_argument(None))?;
        *ptr += len;
        Ok(val)
    }

    pub fn run(&mut self, program: &CompiledProgram) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Running program");
        self.history.clear();
        let mut ptr = 0;
        let mut max_iter = self.max_iter;
        while ptr < program.bytecode.len() {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = program.bytecode[ptr];
            let instr = Instruction::try_from(instr).map_err(|b| {
                warn!(
                    self.logger,
                    "Byte ({}) at {} was not a valid instruction", b, ptr
                );
                ExecutionError::InvalidInstruction(instr)
            })?;
            trace!(
                self.logger,
                "Instruction: {:?}({:?}) Pointer: {:?}",
                instr,
                program.bytecode[ptr],
                ptr
            );
            ptr += 1;
            {
                let nodeid = Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                trace!(self.logger, "Logging visited node {}", nodeid);
                self.history.push(HistoryEntry { id: nodeid, instr });
            }
            match instr {
                Instruction::Start => {}
                Instruction::ClearStack => {
                    self.stack.clear();
                }
                Instruction::SetVar => {
                    let varname: VarName =
                        Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    let scalar = self
                        .stack
                        .pop()
                        .ok_or_else(|| ExecutionError::invalid_argument(None))?;
                    self.variables.insert(varname, scalar);
                }
                Instruction::SetAndSwapVar => {
                    let varname: VarName =
                        Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    let scalar = self.stack.pop().unwrap_or(Scalar::Null);
                    self.variables.insert(varname, scalar);
                    self.stack.push(Scalar::Variable(varname));
                }
                Instruction::ReadVar => {
                    let varname: VarName =
                        Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    let value = self.variables.get(&varname).ok_or_else(|| {
                        debug!(self.logger, "Variable {} does not exist", varname);
                        ExecutionError::invalid_argument(None)
                    })?;
                    self.stack.push(*value);
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or_else(|| {
                        debug!(self.logger, "Value not found");
                        ExecutionError::invalid_argument(None)
                    })?;
                }
                Instruction::Jump => {
                    let label: i32 = Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    ptr = program
                        .labels
                        .get(&label)
                        .ok_or(ExecutionError::InvalidLabel(label))?
                        .block as usize;
                }
                Instruction::Exit => {
                    debug!(self.logger, "Exit called");
                    let code = self.stack.last();
                    if let Some(Scalar::Integer(code)) = code {
                        let code = *code;
                        self.stack.pop();
                        debug!(self.logger, "Exit code {:?}", code);
                        return Ok(code);
                    }
                    return Ok(0);
                }
                Instruction::JumpIfTrue => {
                    self.jump_if(&mut ptr, program, |s| s.as_bool())?;
                }
                Instruction::JumpIfFalse => {
                    self.jump_if(&mut ptr, program, |s| !s.as_bool())?;
                }
                Instruction::CopyLast => {
                    if !self.stack.is_empty() {
                        self.stack.push(self.stack.last().cloned().unwrap());
                    }
                }
                Instruction::Pass => {}
                Instruction::ScalarLabel => {
                    self.stack.push(Scalar::Integer(Self::decode_value(
                        &self.logger,
                        &program.bytecode,
                        &mut ptr,
                    )?));
                }
                Instruction::ScalarInt => {
                    self.stack.push(Scalar::Integer(Self::decode_value(
                        &self.logger,
                        &program.bytecode,
                        &mut ptr,
                    )?));
                }
                Instruction::ScalarFloat => {
                    self.stack.push(Scalar::Floating(Self::decode_value(
                        &self.logger,
                        &program.bytecode,
                        &mut ptr,
                    )?));
                }
                Instruction::ScalarArray => {
                    let len: u32 = Self::decode_value(&self.logger, &program.bytecode, &mut ptr)
                        .and_then(|len: i32| {
                            TryFrom::try_from(len).map_err(|_| {
                                ExecutionError::invalid_argument(
                                    "ScalarArray length must be positive integer".to_owned(),
                                )
                            })
                        })?;
                    if len > 128 || len as usize > self.stack.len() {
                        return Err(ExecutionError::invalid_argument(format!(
                            "The stack holds {} items, but ScalarArray requested {}",
                            self.stack.len(),
                            len,
                        )))?;
                    }
                    let ptr = self.memory.len();
                    for _ in 0..len {
                        let val = self.stack.pop().unwrap();
                        self.write_to_memory(val)?;
                    }
                    self.stack.push(Scalar::Pointer(ptr as i32));
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
                    let literal = Self::read_str(&mut ptr, &program.bytecode)
                        .ok_or(ExecutionError::invalid_argument(None))?;
                    let obj = self.set_value(literal)?;
                    self.stack.push(Scalar::Pointer(obj.index.unwrap() as i32));
                }
                Instruction::Call => self.execute_call(&mut ptr, &program.bytecode)?,
            }
            debug!(
                self.logger,
                "Stack len: {}, last items: {:?}",
                self.stack.len(),
                &self.stack[self.stack.len().max(10) - 10..]
            );
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    fn jump_if<F: Fn(Scalar) -> bool>(
        &mut self,
        ptr: &mut usize,
        program: &CompiledProgram,
        fun: F,
    ) -> Result<(), ExecutionError> {
        if self.stack.len() < 1 {
            warn!(
                self.logger,
                "JumpIfTrue called with missing arguments, stack: {:?}", self.stack
            );
            return Err(ExecutionError::invalid_argument(None));
        }
        let cond = self.stack.pop().unwrap();
        let label: i32 = Self::decode_value(&self.logger, &program.bytecode, ptr)?;
        if fun(cond) {
            *ptr = program
                .labels
                .get(&label)
                .ok_or(ExecutionError::InvalidLabel(label))?
                .block as usize;
        }
        Ok(())
    }

    fn execute_call(&mut self, ptr: &mut usize, bytecode: &[u8]) -> Result<(), ExecutionError> {
        let fun_name = Self::read_str(ptr, bytecode).ok_or_else(|| {
            warn!(self.logger, "Could not read function name");
            ExecutionError::invalid_argument(None)
        })?;
        let mut fun = self
            .callables
            .remove(fun_name.as_str())
            .ok_or_else(|| ExecutionError::ProcedureNotFound(fun_name.as_str().to_owned()))?;
        let res = (|| {
            let n_inputs = fun.num_params();
            let mut inputs = Vec::with_capacity(n_inputs as usize);
            for _ in 0..n_inputs {
                let arg = self.stack.pop().ok_or_else(|| {
                    warn!(
                        self.logger,
                        "Missing argument to function call {:?}", fun_name
                    );
                    ExecutionError::MissingArgument
                })?;
                inputs.push(arg)
            }
            debug!(
                self.logger,
                "Calling function {} with inputs: {:?}", fun_name, inputs
            );
            fun.call(self, &inputs).map_err(|e| {
                warn!(
                    self.logger,
                    "Calling function {:?} failed with {:?}", fun_name, e
                );
                e
            })?;
            debug!(self.logger, "Function call returned");

            Ok(())
        })();
        // clean up
        self.callables.insert(fun_name, fun);
        res
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ExecutionError>
    where
        F: Fn(Scalar, Scalar) -> Scalar,
    {
        let b = pop_stack!(unwrap_var self);
        let a = pop_stack!(unwrap_var self);

        self.stack.push(op(a, b));
        Ok(())
    }

    fn read_str(ptr: &mut usize, program: &[u8]) -> Option<String> {
        let p = *ptr;
        let limit = program.len().min(p + MAX_STR_LEN);
        let s = String::decode(&program[p..limit]).ok()?;
        *ptr += s.len() + i32::BYTELEN;
        Some(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let value: TPointer = 12342;
        let encoded = value.encode().unwrap();
        let decoded = TPointer::decode(&encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_set_value_memory_limit_error_raised() {
        let mut vm = VM::new(None, ());
        vm.memory_limit = 10;
        vm.set_value("1234567890987654321".to_owned())
            .expect_err("Should return error");
    }

    #[test]
    fn test_array_literal_memory_limit_error_raised() {
        let program = r#"{
  "nodes": {
    "0": {
      "node": {
        "Start": null
      },
      "child": 1
    },
    "1": {
      "node": {
        "ScalarInt": 42
      },
      "child": 2
    },
    "2": {
      "node": {
        "ScalarInt": 42
      },
      "child": 3
    },
    "3": {
      "node": {
        "ScalarInt": 42
      },
      "child": 30
    },
    "30": {
      "node": {
        "ScalarArray": 3
      }
    }
  }
}
            "#;

        let compilation_unit = serde_json::from_str(program).unwrap();
        let program = crate::compiler::compile(None, compilation_unit).unwrap();

        let mut vm = VM::new(None, ());
        vm.memory_limit = 8;

        let err = vm.run(&program).expect_err("Should have failed");

        match err {
            ExecutionError::OutOfMemory => {}
            _ => panic!("Expected out of memory {:?}", err),
        }
    }

    #[test]
    fn test_binary_operatons() {
        let mut vm = VM::new(None, ());

        vm.stack.push(Scalar::Integer(512));
        vm.stack.push(Scalar::Integer(42));

        vm.binary_op(|a, b| (a + a / b) * b).unwrap();

        let result = vm.stack.last().expect("Expected to read the result");
        match result {
            Scalar::Integer(result) => assert_eq!(*result, (512 + 512 / 42) * 42),
            _ => panic!("Invalid result type"),
        }
    }

    #[test]
    fn test_str_get() {
        let mut vm = VM::new(None, ());

        let obj = vm.set_value("winnie".to_owned()).unwrap();
        let ind = obj.index.unwrap();

        let val1 = vm.get_value_in_place::<&str>(ind).unwrap();
        let val2 = vm.get_value_in_place::<&str>(ind).unwrap();

        assert_eq!(val1, val2);
        assert_eq!(val1, "winnie");
    }
}
