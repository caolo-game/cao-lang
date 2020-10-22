pub mod data;

use crate::{binary_compare, pop_stack};
use crate::{collections::pre_hash_map::Key, instruction::Instruction};
use crate::{collections::pre_hash_map::PreHashMap, prelude::*};
use crate::{collections::stack::ScalarStack, VariableId};
use crate::{scalar::Scalar, InputString};
use data::RuntimeData;
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

    pub runtime_data: RuntimeData,

    callables: PreHashMap<Procedure<Aux>>,
    objects: HashMap<Pointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<Pointer, ConvertFn<Aux>>,
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
            callables: PreHashMap::default(),
            objects: HashMap::with_capacity(128),
            runtime_data: RuntimeData {
                memory_limit: 40000,
                memory: Vec::with_capacity(512),
                stack: ScalarStack::new(256),
                registers: Vec::with_capacity(128),
            },
            max_iter: 1000,
            _m: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.converters.clear();

        self.runtime_data.clear();
    }

    pub fn read_var(&self, name: VariableId) -> Option<&Scalar> {
        self.runtime_data.registers.get(name.0 as usize)
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_iter = max_iter;
        self
    }

    pub fn stack(&self) -> &[Scalar] {
        self.runtime_data.stack.as_slice()
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

    pub fn register_function<C: Callable<Aux> + 'static>(&mut self, name: InputString, f: C) {
        let hash = Key::from_str(name.as_str());
        self.callables.insert(hash, Procedure::new(name, f));
    }

    pub fn register_function_obj(&mut self, name: &str, f: Procedure<Aux>) {
        let hash = Key::from_str(name);
        self.callables.insert(hash, f);
    }

    pub fn get_value_in_place<T: DecodeInPlace<'a>>(
        &'a self,
        bytecode_pos: Pointer,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
        let object = self.objects.get(&bytecode_pos)?;

        match self.runtime_data.get_value_in_place::<T>(object) {
            Some(val) => Some(val),
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, bytecode_pos: Pointer) -> Option<T> {
        let object = self.objects.get(&bytecode_pos)?;
        match object.index {
            Some(index) => {
                let data = &self.runtime_data.memory;
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

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value_with_decoder<T: ByteEncodeProperties>(
        &mut self,
        val: T,
        converter: ConvertFn<Aux>,
    ) -> Result<Object, ExecutionError> {
        let (index, size) = self.runtime_data.write_to_memory(val)?;
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
        let (index, size) = self.runtime_data.write_to_memory(val)?;
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
        self.runtime_data
            .stack
            .push(value.into())
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Option<Scalar> {
        self.runtime_data.stack.pop()
    }

    #[inline]
    unsafe fn decode_value<T: ByteDecodeProperties>(
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
    fn decode_in_place<T: DecodeInPlace<'a>>(
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

    /// This mostly assumes that program is valid, produced by the compiler.
    /// As such running non-compiler emitted programs is fairly unsafe
    pub fn run(&mut self, program: &'a CompiledProgram) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Running program");
        self.history.clear();
        let mut bytecode_pos = 0;
        let len = program.bytecode.len();
        let mut max_iter = self.max_iter;
        while bytecode_pos < len {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = unsafe { *program.bytecode.as_ptr().offset(bytecode_pos as isize) };
            let instr = unsafe { transmute(instr) };
            trace!(
                self.logger,
                "Instruction: {:?}({:?}) Pointer: {:?}",
                instr,
                program.bytecode[bytecode_pos],
                bytecode_pos
            );
            bytecode_pos += 1;
            match instr {
                Instruction::Breadcrumb => {
                    self.instr_breadcrumb(&program.bytecode, &mut bytecode_pos)
                }
                Instruction::ClearStack => {
                    self.runtime_data.stack.clear();
                }
                Instruction::SetVar => {
                    self.instr_set_var(&program.bytecode, &mut bytecode_pos)?;
                }
                Instruction::ReadVar => {
                    self.instr_read_var(&program.bytecode, &mut bytecode_pos)?;
                }
                Instruction::Pop => {
                    self.runtime_data.stack.pop().ok_or_else(|| {
                        debug!(self.logger, "Popping empty stack");
                        ExecutionError::invalid_argument(Some("Popping empty stack".to_owned()))
                    })?;
                }
                Instruction::Jump => {
                    self.instr_jump(&mut bytecode_pos, program)?;
                }
                Instruction::Exit => return self.instr_exit(),
                Instruction::JumpIfTrue => {
                    self.jump_if(&mut bytecode_pos, program, |s| s.as_bool())?;
                }
                Instruction::JumpIfFalse => {
                    self.jump_if(&mut bytecode_pos, program, |s| !s.as_bool())?;
                }
                Instruction::CopyLast => {
                    if let Some(val) = self.runtime_data.stack.last().cloned() {
                        self.runtime_data
                            .stack
                            .push(val)
                            .map_err(|_| ExecutionError::Stackoverflow)?;
                    }
                }
                Instruction::Pass => {}
                Instruction::ScalarLabel => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            Self::decode_value(&self.logger, &program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarInt => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Integer(unsafe {
                            Self::decode_value(&self.logger, &program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarFloat => {
                    self.runtime_data
                        .stack
                        .push(Scalar::Floating(unsafe {
                            Self::decode_value(&self.logger, &program.bytecode, &mut bytecode_pos)
                        }))
                        .map_err(|_| ExecutionError::Stackoverflow)?;
                }
                Instruction::ScalarArray => {
                    self.instr_scalar_array(&program.bytecode, &mut bytecode_pos)?
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
                    self.instr_string_literal(&mut bytecode_pos, &program.bytecode)?
                }
                Instruction::Call => self.execute_call(&mut bytecode_pos, &program.bytecode)?,
            }
            debug!(
                self.logger,
                "Stack len: {} {:?} ptr: {}",
                self.runtime_data.stack.len(),
                self.log_stack(),
                bytecode_pos
            );
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    fn instr_read_var(&mut self, bytecode: &'a [u8], bytecode_pos: &mut usize) -> ExecutionResult {
        let VariableId(varname) =
            unsafe { Self::decode_value(&self.logger, bytecode, bytecode_pos) };
        let value = self
            .runtime_data
            .registers
            .get(varname as usize)
            .ok_or_else(|| {
                debug!(self.logger, "Variable {} does not exist", varname);
                ExecutionError::invalid_argument(None)
            })?;
        self.runtime_data
            .stack
            .push(*value)
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    fn instr_set_var(&mut self, bytecode: &'a [u8], bytecode_pos: &mut usize) -> ExecutionResult {
        let varname =
            unsafe { Self::decode_value::<VariableId>(&self.logger, bytecode, bytecode_pos) };
        let scalar = self.runtime_data.stack.pop().ok_or_else(|| {
            ExecutionError::invalid_argument("Stack was empty when setting variable".to_owned())
        })?;
        let varname = varname.0 as usize;
        if self.runtime_data.registers.len() <= varname {
            self.runtime_data
                .registers
                .resize(varname + 1, Scalar::Null);
        }
        self.runtime_data.registers[varname] = scalar;
        Ok(())
    }

    fn instr_exit(&mut self) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Exit called");
        let code = self.runtime_data.stack.pop();
        if let Some(Scalar::Integer(code)) = code {
            debug!(self.logger, "Exit code {:?}", code);
            return Ok(code);
        }
        Ok(0)
    }

    fn instr_breadcrumb(&mut self, bytecode: &'a [u8], bytecode_pos: &mut usize) {
        let nodeid = unsafe { Self::decode_value(&self.logger, &bytecode, bytecode_pos) };

        let instr = bytecode[*bytecode_pos];
        let instr = Instruction::try_from(instr).ok();
        *bytecode_pos += 1;
        trace!(self.logger, "Logging visited node {:?}", nodeid);
        self.history.push(HistoryEntry { id: nodeid, instr });
    }

    fn instr_scalar_array(
        &mut self,
        bytecode: &'a [u8],
        bytecode_pos: &mut usize,
    ) -> ExecutionResult {
        let len: i32 = unsafe { Self::decode_value(&self.logger, bytecode, bytecode_pos) };
        if len < 0 {
            return Err(ExecutionError::invalid_argument(
                "ScalarArray length must be positive integer".to_owned(),
            ));
        }
        if len as usize > self.runtime_data.stack.len() {
            return Err(ExecutionError::invalid_argument(format!(
                "The stack holds {} items, but ScalarArray requested {}",
                self.runtime_data.stack.len(),
                len,
            )));
        }
        let bytecode_pos = self.runtime_data.memory.len();
        for _ in 0..len {
            if let Some(val) = self.runtime_data.stack.pop() {
                self.runtime_data.write_to_memory(val)?;
            }
        }
        self.runtime_data
            .stack
            .push(Scalar::Pointer(Pointer(bytecode_pos as u32)))
            .map_err(|_| ExecutionError::Stackoverflow)?;

        Ok(())
    }

    fn instr_string_literal(
        &mut self,
        bytecode_pos: &mut usize,
        bytecode: &'a [u8],
    ) -> ExecutionResult {
        let literal = Self::read_str(bytecode_pos, bytecode)
            .ok_or_else(|| ExecutionError::invalid_argument(None))?;
        let obj = self.set_value_with_decoder(literal, |o, vm| {
            // SAFETY
            // As long as the same VM instance's accessors are used this should be
            // fine (tm)
            let res = vm.get_value_in_place::<&str>(o.index.unwrap()).unwrap();
            let res: &'static str = unsafe { mem::transmute(res) };
            Box::new(res)
        })?;
        self.runtime_data
            .stack
            .push(Scalar::Pointer(obj.index.unwrap()))
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    fn instr_jump(
        &mut self,
        bytecode_pos: &mut usize,
        program: &CompiledProgram,
    ) -> ExecutionResult {
        let label: Key =
            unsafe { Self::decode_value(&self.logger, &program.bytecode, bytecode_pos) };
        *bytecode_pos = program
            .labels
            .0
            .get(label)
            .ok_or(ExecutionError::InvalidLabel(label))?
            .pos as usize;
        Ok(())
    }

    pub fn log_stack(&self) {
        trace!(self.logger, "--------Stack--------");
        for s in self.runtime_data.stack.as_slice().iter().rev() {
            trace!(self.logger, "{:?}", s);
        }
        trace!(self.logger, "------End Stack------");
    }

    fn jump_if<F: Fn(Scalar) -> bool>(
        &mut self,
        bytecode_pos: &mut usize,
        program: &CompiledProgram,
        predicate: F,
    ) -> Result<(), ExecutionError> {
        if self.runtime_data.stack.is_empty() {
            warn!(
                self.logger,
                "JumpIfTrue called with missing arguments, stack: {:?}", self.runtime_data.stack
            );
            return Err(ExecutionError::invalid_argument(None));
        }
        let cond = self.runtime_data.stack.pop().unwrap();
        let label: Key =
            unsafe { Self::decode_value(&self.logger, &program.bytecode, bytecode_pos) };
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

    fn execute_call(
        &mut self,
        bytecode_pos: &mut usize,
        bytecode: &'a [u8],
    ) -> Result<(), ExecutionError> {
        let fun_hash = unsafe { Self::decode_value(&self.logger, bytecode, bytecode_pos) };
        let mut fun = self
            .callables
            .remove(fun_hash)
            .ok_or_else(|| ExecutionError::ProcedureNotFound(fun_hash))?;
        let logger = self.logger.new(o!("function"=> fun.name.to_string()));
        let res = (|| {
            let n_inputs = fun.num_params();
            let mut inputs = Vec::with_capacity(n_inputs as usize);
            for i in 0..n_inputs {
                let arg = self.runtime_data.stack.pop().ok_or_else(|| {
                    warn!(logger, "Missing argument ({}) to function call", i);
                    ExecutionError::MissingArgument
                })?;
                inputs.push(arg)
            }
            debug!(logger, "Calling function with inputs: {:?}", inputs);
            fun.call(self, &inputs).map_err(|e| {
                warn!(logger, "Calling function failed with {:?}", e);
                e
            })?;
            debug!(logger, "Function call returned");

            Ok(())
        })();
        // cleanup
        self.callables.insert(fun_hash, fun);
        res
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ExecutionError>
    where
        F: Fn(Scalar, Scalar) -> Scalar,
    {
        let b = pop_stack!(self);
        let a = pop_stack!(self);

        self.runtime_data
            .stack
            .push(op(a, b))
            .map_err(|_| ExecutionError::Stackoverflow)?;
        Ok(())
    }

    fn read_str(bytecode_pos: &mut usize, program: &'a [u8]) -> Option<&'a str> {
        let p = *bytecode_pos;
        let limit = program.len().min(p + MAX_STR_LEN);
        let (len, s): (_, &'a str) =
            <&'a str as DecodeInPlace>::decode_in_place(&program[p..limit]).ok()?;
        *bytecode_pos += len;
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
        vm.runtime_data.memory_limit = 10;
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
        vm.runtime_data.memory_limit = 8;

        let err = vm.run(&program).expect_err("Should have failed");

        match err {
            ExecutionError::OutOfMemory => {}
            _ => panic!("Expected out of memory {:?}", err),
        }
    }

    #[test]
    fn test_binary_operatons() {
        let mut vm = VM::new(None, ());

        vm.runtime_data.stack.push(Scalar::Integer(512)).unwrap();
        vm.runtime_data.stack.push(Scalar::Integer(42)).unwrap();

        vm.binary_op(|a, b| (a + a / b) * b).unwrap();

        let result = vm
            .runtime_data
            .stack
            .pop()
            .expect("Expected to read the result");
        match result {
            Scalar::Integer(result) => assert_eq!(result, (512 + 512 / 42) * 42),
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
