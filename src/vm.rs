use crate::compiler::NodeId;
use crate::instruction::Instruction;
use crate::prelude::*;
use crate::scalar::Scalar;
use crate::VarName;
use crate::{binary_compare, pop_stack};
use slog::{debug, error, warn};
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

/// Cao-Lang bytecode interpreter.
/// Aux is an auxiliary data structure passed to custom functions.
pub struct VM<Aux = ()> {
    pub logger: Logger,

    memory: Vec<u8>,
    stack: Vec<Scalar>,
    callables: HashMap<String, Procedure<Aux>>,
    objects: HashMap<TPointer, Object>,
    /// Functions to convert Objects to dyn ObjectProperties
    converters: HashMap<TPointer, ConvertFn<Aux>>,
    variables: HashMap<VarName, Scalar>,
    auxiliary_data: Aux,
    max_iter: i32,
    memory_limit: usize,
}

impl<Aux> VM<Aux> {
    pub fn new(logger: impl Into<Option<Logger>>, auxiliary_data: Aux) -> Self {
        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        Self {
            logger,
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
                let tail = (head + size as usize).min(data.len());
                T::decode(&data[head..tail]).ok()
            }
            None => {
                warn!(self.logger, "Dereferencing null pointer");
                None
            }
        }
    }

    /// Save `val` in memory and push a pointer to the object onto the stack
    pub fn set_value<T: ByteEncodeProperties + 'static>(
        &mut self,
        val: T,
    ) -> Result<Object, ExecutionError> {
        let result = self.memory.len();
        let bytes = val.encode().map_err(|err|
            {
                warn!(self.logger, "Failed to encode argument {:?}", err);
                ExecutionError::InvalidArgument
            })?;

        if bytes.len() + result >= self.memory_limit {
            return Err(ExecutionError::OutOfMemory);
        }

        let object = Object {
            index: Some(result as i32),
            size: T::BYTELEN as u32,
        };
        self.memory.extend(bytes.iter());

        self.objects.insert(result as TPointer, object);
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

    pub fn run(&mut self, program: &CompiledProgram) -> Result<i32, ExecutionError> {
        debug!(self.logger, "Running program");
        let mut ptr = 0;
        let mut max_iter = self.max_iter;
        while ptr < program.bytecode.len() {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = Instruction::try_from(program.bytecode[ptr]).map_err(|_| {
                error!(
                    self.logger,
                    "Byte at {}: {:?} was not a valid instruction", ptr, program.bytecode[ptr]
                );
                ExecutionError::InvalidInstruction
            })?;
            debug!(
                self.logger,
                "Instruction: {:?}({:?}) Pointer: {:?}", instr, program.bytecode[ptr], ptr
            );
            ptr += 1;
            match instr {
                Instruction::Start => {}
                Instruction::ClearStack => {
                    self.stack.clear();
                }
                Instruction::SetVar => {
                    let len = VarName::BYTELEN;
                    let varname = VarName::decode(&program.bytecode[ptr..ptr + len])
                        .map_err(|_| ExecutionError::InvalidArgument)?;
                    ptr += len;
                    let scalar = self
                        .stack
                        .pop()
                        .ok_or_else(|| ExecutionError::InvalidArgument)?;
                    self.variables.insert(varname, scalar);
                }
                Instruction::SetAndSwapVar => {
                    let len = VarName::BYTELEN;
                    let varname = VarName::decode(&program.bytecode[ptr..ptr + len])
                        .map_err(|_| ExecutionError::InvalidArgument)?;
                    ptr += len;
                    let scalar = self.stack.pop().unwrap_or(Scalar::Null);
                    self.variables.insert(varname, scalar);
                    self.stack.push(Scalar::Variable(varname));
                }
                Instruction::ReadVar => {
                    let len = VarName::BYTELEN;
                    let varname = VarName::decode(&program.bytecode[ptr..ptr + len])
                        .map_err(|_| ExecutionError::InvalidArgument)?;
                    ptr += len;
                    let value = self.variables.get(&varname).ok_or_else(|| {
                        debug!(self.logger, "Variable {} does not exist", varname);
                        ExecutionError::InvalidArgument
                    })?;
                    self.stack.push(*value);
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or_else(|| {
                        debug!(self.logger, "Value not found");
                        ExecutionError::InvalidArgument
                    })?;
                }
                Instruction::Jump => {
                    let len = i32::BYTELEN;
                    let bytes = &program.bytecode[ptr..ptr + len];
                    let label = i32::decode(bytes).map_err(|_| ExecutionError::InvalidLabel)?;
                    ptr = program
                        .labels
                        .get(&label)
                        .ok_or(ExecutionError::InvalidLabel)?[0] as usize;
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
                    if self.stack.len() < 1 {
                        error!(
                            self.logger,
                            "JumpIfTrue called with missing arguments, stack: {:?}", self.stack
                        );
                        return Err(ExecutionError::InvalidArgument);
                    }
                    let cond = self.stack.pop().unwrap();
                    let len = i32::BYTELEN;
                    let label = i32::decode(&program.bytecode[ptr..ptr + len])
                        .map_err(|_| ExecutionError::InvalidLabel)?;
                    if cond.as_bool() {
                        ptr = program
                            .labels
                            .get(&label)
                            .ok_or(ExecutionError::InvalidLabel)?[0]
                            as usize;
                    } else {
                        ptr += len;
                    }
                }
                Instruction::CopyLast => {
                    if !self.stack.is_empty() {
                        self.stack.push(self.stack.last().cloned().unwrap());
                    }
                }
                Instruction::Pass => {}
                Instruction::ScalarLabel => {
                    let len = NodeId::BYTELEN;
                    self.stack.push(Scalar::Integer(
                        NodeId::decode(&program.bytecode[ptr..ptr + len])
                            .map_err(|_| ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarInt => {
                    let len = i32::BYTELEN;
                    self.stack.push(Scalar::Integer(
                        i32::decode(&program.bytecode[ptr..ptr + len])
                            .map_err(|_| ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarFloat => {
                    let len = f32::BYTELEN;
                    self.stack.push(Scalar::Floating(
                        f32::decode(&program.bytecode[ptr..ptr + len])
                            .map_err(|_| ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarArray => {
                    let len = self
                        .load_ptr_from_stack()
                        .ok_or(ExecutionError::InvalidArgument)?;
                    if len > 128 || len > self.stack.len() as i32 {
                        return Err(ExecutionError::InvalidArgument)?;
                    }
                    let ptr = self.memory.len();
                    self.stack.pop();
                    for _ in 0..len {
                        let val = self.stack.pop().unwrap();
                        self.memory.append(&mut val.encode().map_err(|err| {
                            error!(self.logger, "Failed to encode array {:?}", err);
                            ExecutionError::InvalidArgument
                        })?);
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
                        .ok_or(ExecutionError::InvalidArgument)?;
                    let obj = self.set_value(literal)?;
                    self.stack.push(Scalar::Pointer(obj.index.unwrap() as i32));
                }
                Instruction::Call => self.execute_call(&mut ptr, &program.bytecode)?,
            }
            if self.memory.len() > self.memory_limit {
                return Err(ExecutionError::OutOfMemory);
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

    fn execute_call(&mut self, ptr: &mut usize, bytecode: &[u8]) -> Result<(), ExecutionError> {
        let fun_name = Self::read_str(ptr, bytecode).ok_or_else(|| {
            error!(self.logger, "Could not read function name");
            ExecutionError::InvalidArgument
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
                    error!(
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
                error!(
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

    fn load_ptr_from_stack(&self) -> Option<i32> {
        let val = self.stack.last()?;
        match val {
            Scalar::Pointer(i) => Some(*i),
            _ => None,
        }
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
    use crate::procedures::FunctionWrapper;
    use std::str::FromStr;

    #[test]
    fn test_encode() {
        let value: TPointer = 12342;
        let encoded = value.encode().unwrap();
        let decoded = TPointer::decode(&encoded).unwrap();

        assert_eq!(value, decoded);
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
    fn test_simple_program() {
        let mut bytecode = Vec::with_capacity(512);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 512i32.encode().unwrap());
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 42i32.encode().unwrap());
        bytecode.push(Instruction::Sub as u8);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 68i32.encode().unwrap());
        bytecode.push(Instruction::Add as u8);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 0i32.encode().unwrap());
        bytecode.push(Instruction::Exit as u8);
        let mut program = CompiledProgram::default();
        program.bytecode = bytecode;

        let mut vm = VM::new(None, ());
        vm.run(&program).unwrap();
        assert_eq!(vm.stack.len(), 1);
        let value = vm.stack.last().unwrap();
        match value {
            Scalar::Integer(i) => assert_eq!(*i, 512 - 42 + 68),
            _ => panic!("Invalid value in the stack"),
        }
    }

    #[test]
    fn test_function_call() {
        let mut bytecode = Vec::with_capacity(512);
        bytecode.push(Instruction::ScalarFloat as u8);
        bytecode.append(&mut 42.0f32.encode().unwrap());
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 512i32.encode().unwrap());
        bytecode.push(Instruction::Call as u8);
        bytecode.append(&mut "foo".to_owned().encode().unwrap());
        bytecode.push(Instruction::Exit as u8);

        let mut program = CompiledProgram::default();
        program.bytecode = bytecode;

        let mut vm = VM::new(None, ());

        fn foo(vm: &mut VM<()>, (a, b): (i32, f32)) -> ExecutionResult {
            let res = a as f32 * b % 13.;
            let res = res as i32;

            vm.set_value(res)?;
            Ok(())
        };

        vm.register_function("foo", FunctionWrapper::new(foo));
        vm.register_function(
            "bar",
            FunctionWrapper::new(|_vm: &mut VM, _a: i32| {
                Err::<(), _>(ExecutionError::Unimplemented)
            }),
        );

        vm.run(&program).unwrap();

        let ptr = 0;
        let res = vm.get_value::<i32>(ptr).unwrap();

        assert_eq!(res, ((512. as f32) * (42. as f32) % 13.) as i32);
    }

    #[test]
    fn test_variable_scalar() {
        let mut bytecode = Vec::with_capacity(512);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 420i32.encode().unwrap());
        bytecode.push(Instruction::Equals as u8);
        bytecode.push(Instruction::Exit as u8);

        let mut program = CompiledProgram::default();
        program.bytecode = bytecode;

        let mut vm = VM::new(None, ());
        vm.variables
            .insert(VarName::from_str("foo").unwrap(), Scalar::Integer(69));
        vm.stack
            .push(Scalar::Variable(VarName::from_str("foo").unwrap()));

        let res = vm.run(&program).unwrap();

        assert_eq!(res, 0);
    }
}
