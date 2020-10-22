use crate::{collections::pre_hash_map::Key, instruction::Instruction};
use crate::prelude::*;
use crate::scalar::Scalar;
use crate::VariableId;
use crate::{binary_compare, pop_stack};
use serde::{Deserialize, Serialize};
use slog::{debug, trace, warn};
use slog::{o, Drain, Logger};
use std::{collections::HashMap, mem};
use std::{convert::TryFrom, mem::transmute};

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
    pub index: Option<Pointer>,
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

    pub fn as_inner<Aux>(&self, vm: &VM<Aux>) -> Result<Box<dyn ObjectProperties>, ConvertError> {
        self.index
            .ok_or(ConvertError::NullPtr)
            .map(|index| unsafe { vm.converters[&index](self, vm) })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub struct HistoryEntry {
    pub id: NodeId,
    pub instr: Option<Instruction>,
}

/// Cao-Lang bytecode interpreter.
/// `Aux` is an auxiliary data structure passed to custom functions.
pub struct VM<'a, Aux = ()>
where
    Aux: 'a,
{
    pub logger: Logger,
    pub history: Vec<HistoryEntry>,
    pub auxiliary_data: Aux,
    pub max_iter: i32,
    pub memory_limit: usize,

    memory: Vec<u8>,
    stack: Vec<Scalar>,
    callables: HashMap<String, Procedure<Aux>>,
    objects: HashMap<Pointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<Pointer, ConvertFn<Aux>>,
    variables: Vec<Scalar>,
    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, Aux> VM<'a, Aux> {
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
            variables: Vec::with_capacity(128),
            max_iter: 1000,
            _m: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.memory.clear();
        self.stack.clear();
        self.objects.clear();
        self.converters.clear();
        self.variables.clear();
    }

    pub fn read_var(&self, name: VariableId) -> Option<&Scalar> {
        self.variables.get(name.0 as usize)
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

    pub fn get_value_in_place<T: DecodeInPlace<'a>>(
        &'a self,
        ptr: Pointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        let object = self.objects.get(&ptr)?;
        match object.index {
            Some(index) => {
                let data = &self.memory;
                let head = index.0 as usize;
                let tail = (head.checked_add(object.size as usize))
                    .unwrap_or(data.len())
                    .min(data.len());
                T::decode_in_place(&data[head..tail])
                    .ok()
                    .map(|(_, val)| val)
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, ptr: Pointer) -> Option<T> {
        let object = self.objects.get(&ptr)?;
        match object.index {
            Some(index) => {
                let data = &self.memory;
                let head = index.0 as usize;
                let tail = (head.checked_add(object.size as usize))
                    .unwrap_or(data.len())
                    .min(data.len());
                T::decode(&data[head..tail]).ok().map(|(_, val)| val)
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    fn write_to_memory<T: ByteEncodeProperties>(
        &mut self,
        val: T,
    ) -> Result<(Pointer, usize), ExecutionError> {
        let result = self.memory.len();

        val.encode(&mut self.memory).map_err(|err| {
            warn!(self.logger, "Failed to encode argument {:?}", err);
            ExecutionError::invalid_argument(None)
        })?;

        if self.memory.len() >= self.memory_limit {
            return Err(ExecutionError::OutOfMemory);
        }
        Ok((Pointer(result as u32), self.memory.len() - result))
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value_with_decoder<T: ByteEncodeProperties>(
        &mut self,
        val: T,
        converter: ConvertFn<Aux>,
    ) -> Result<Object, ExecutionError> {
        let (index, size) = self.write_to_memory(val)?;
        let object = Object {
            index: Some(index),
            size: size as u32,
        };
        self.objects.insert(index, object);
        self.converters.insert(index as Pointer, converter);

        self.stack_push(Scalar::Pointer(index as Pointer))?;

        debug!(self.logger, "Set value {:?} {}", object, T::displayname());

        Ok(object)
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value<T: ByteEncodeProperties + ByteDecodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<Object, ExecutionError> {
        let (index, size) = self.write_to_memory(val)?;
        let object = Object {
            index: Some(index),
            size: size as u32,
        };
        self.objects.insert(index, object);
        self.converters
            .insert(index as Pointer, |o: &Object, vm: &VM<Aux>| {
                let res: T = vm.get_value(o.index.unwrap()).unwrap();
                Box::new(res)
            });

        self.stack_push(Scalar::Pointer(index as Pointer))?;

        debug!(self.logger, "Set value {:?} {}", object, T::displayname());

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
    fn decode_value<T: ByteDecodeProperties>(
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
        let (len, val) = T::decode(&bytes[*ptr..])
            .map_err(|_| ExecutionError::invalid_argument("Failed to decode value".to_owned()))?;
        *ptr += len;
        Ok(val)
    }

    #[allow(unused)]
    #[inline]
    fn decode_in_place<T: DecodeInPlace<'a>>(
        logger: &Logger,
        bytes: &'a [u8],
        ptr: &mut usize,
    ) -> Result<T::Ref, ExecutionError> {
        trace!(
            logger,
            "Decoding value of type {} at ptr {}, len: {}",
            std::any::type_name::<T>(),
            ptr,
            bytes.len()
        );
        let (len, val) = T::decode_in_place(&bytes[*ptr..])
            .map_err(|_| ExecutionError::invalid_argument("Failed to decode value".to_owned()))?;
        *ptr += len;
        trace!(logger, "Decoding successful, new ptr {}", ptr,);
        Ok(val)
    }

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is fairly unsafe
    pub fn run(&mut self, program: &'a CompiledProgram) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Running program");
        self.history.clear();
        let mut ptr = 0;
        let mut max_iter = self.max_iter;
        while ptr < program.bytecode.len() {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = unsafe { *program.bytecode.as_ptr().offset(ptr as isize) };
            let instr = unsafe { transmute(instr) };
            trace!(
                self.logger,
                "Instruction: {:?}({:?}) Pointer: {:?}",
                instr,
                program.bytecode[ptr],
                ptr
            );
            ptr += 1;
            match instr {
                Instruction::Breadcrumb => {
                    let nodeid = Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    let instr = program.bytecode[ptr];
                    let instr = Instruction::try_from(instr).ok();
                    ptr += 1;
                    trace!(self.logger, "Logging visited node {:?}", nodeid);
                    self.history.push(HistoryEntry { id: nodeid, instr });
                }
                Instruction::ClearStack => {
                    self.stack.clear();
                }
                Instruction::SetVar => {
                    let varname = Self::decode_value::<VariableId>(
                        &self.logger,
                        &program.bytecode,
                        &mut ptr,
                    )?;
                    let scalar = self.stack.pop().ok_or_else(|| {
                        ExecutionError::invalid_argument(
                            "Stack was empty when setting variable".to_owned(),
                        )
                    })?;
                    let varname = varname.0 as usize;
                    if self.variables.len() <= varname {
                        self.variables.resize(varname + 1, Scalar::Null);
                    }
                    self.variables[varname] = scalar;
                }
                Instruction::ReadVar => {
                    let VariableId(varname) =
                        Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    let value = self.variables.get(varname as usize).ok_or_else(|| {
                        debug!(self.logger, "Variable {} does not exist", varname);
                        ExecutionError::invalid_argument(None)
                    })?;
                    self.stack.push(*value);
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or_else(|| {
                        debug!(self.logger, "Popping empty stack");
                        ExecutionError::invalid_argument(Some("Popping empty stack".to_owned()))
                    })?;
                }
                Instruction::Jump => {
                    let label: Key =
                        Self::decode_value(&self.logger, &program.bytecode, &mut ptr)?;
                    ptr = program
                        .labels
                        .0
                        .get(label)
                        .ok_or(ExecutionError::InvalidLabel(label))?
                        .pos as usize;
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
                    if let Some(val) = self.stack.last().cloned() {
                        self.stack.push(val);
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
                        )));
                    }
                    let ptr = self.memory.len();
                    for _ in 0..len {
                        if let Some(val) = self.stack.pop() {
                            self.write_to_memory(val)?;
                        }
                    }
                    self.stack.push(Scalar::Pointer(Pointer(ptr as u32)));
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
                        .ok_or_else(|| ExecutionError::invalid_argument(None))?;
                    let obj = self.set_value_with_decoder(literal, |o, vm| {
                        // SAFETY
                        // As long as the same VM instance's accessors are used this should be
                        // fine (tm)
                        let res = vm.get_value_in_place::<&str>(o.index.unwrap()).unwrap();
                        let res: &'static str = unsafe { mem::transmute(res) };
                        Box::new(res)
                    })?;
                    self.stack.push(Scalar::Pointer(obj.index.unwrap()));
                }
                Instruction::Call => self.execute_call(&mut ptr, &program.bytecode)?,
            }
            debug!(
                self.logger,
                "Stack len: {} {:?}",
                self.stack.len(),
                self.log_stack()
            );
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    pub fn log_stack(&self) {
        trace!(self.logger, "--------Stack--------");
        for s in &self.stack[..] {
            trace!(self.logger, "{:?}", s);
        }
        trace!(self.logger, "------End Stack------");
    }

    fn jump_if<F: Fn(Scalar) -> bool>(
        &mut self,
        ptr: &mut usize,
        program: &CompiledProgram,
        predicate: F,
    ) -> Result<(), ExecutionError> {
        if self.stack.is_empty() {
            warn!(
                self.logger,
                "JumpIfTrue called with missing arguments, stack: {:?}", self.stack
            );
            return Err(ExecutionError::invalid_argument(None));
        }
        let cond = self.stack.pop().unwrap();
        let label: Key = Self::decode_value(&self.logger, &program.bytecode, ptr)?;
        if predicate(cond) {
            *ptr = program
                .labels
                .0
                .get(label)
                .ok_or(ExecutionError::InvalidLabel(label))?
                .pos as usize;
        }
        Ok(())
    }

    fn execute_call(&mut self, ptr: &mut usize, bytecode: &'a [u8]) -> Result<(), ExecutionError> {
        let fun_name = Self::read_str(ptr, bytecode).ok_or_else(|| {
            warn!(self.logger, "Could not read function name");
            ExecutionError::invalid_argument(None)
        })?;
        let mut fun = self
            .callables
            .remove(fun_name)
            .ok_or_else(|| ExecutionError::ProcedureNotFound(fun_name.to_owned()))?;
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
        self.callables.insert(fun_name.to_owned(), fun);
        res
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ExecutionError>
    where
        F: Fn(Scalar, Scalar) -> Scalar,
    {
        let b = pop_stack!(self);
        let a = pop_stack!(self);

        self.stack.push(op(a, b));
        Ok(())
    }

    fn read_str(ptr: &mut usize, program: &'a [u8]) -> Option<&'a str> {
        let p = *ptr;
        let limit = program.len().min(p + MAX_STR_LEN);
        let (len, s): (_, &'a str) =
            <&'a str as DecodeInPlace>::decode_in_place(&program[p..limit]).ok()?;
        *ptr += len;
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let value = Pointer(12342);
        let mut encoded = Vec::new();
        value.encode(&mut encoded).unwrap();
        let (_, decoded) = Pointer::decode(&encoded).unwrap();

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
  "lanes": [ {
    "name": "Foo",
    "cards": [
        { "ScalarInt": 42 },
        { "ScalarInt": 42 },
        { "ScalarInt": 42 },
        { "ScalarArray": 3 }
    ]
  } ]
}
            "#;

        let compilation_unit = serde_json::from_str(program).unwrap();
        let program = crate::compiler::compile(None, compilation_unit, None).unwrap();

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

    #[test]
    fn test_str_get_drop() {
        let mut vm = VM::new(None, ());

        let obj = vm.set_value("winnie".to_owned()).unwrap();
        let ind = obj.index.unwrap();

        {
            let _val1 = vm.get_value_in_place::<&str>(ind).unwrap();
        }

        let val2 = vm.get_value_in_place::<&str>(ind).unwrap();

        assert_eq!(val2, "winnie");
    }
}
