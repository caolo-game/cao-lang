use crate::{
    alloc::BumpProxy,
    collections::{bounded_stack::BoundedStack, value_stack::ValueStack},
};
use crate::{
    alloc::{Allocator, BumpAllocator},
    collections::key_map::KeyMap,
};
use crate::{prelude::*, value::Value};

pub type Tables = KeyMap<FieldTable, BumpProxy>;
pub type FieldTable = KeyMap<Value, BumpProxy>;

pub struct RuntimeData {
    pub(crate) stack: ValueStack,
    pub(crate) call_stack: BoundedStack<CallFrame>,
    pub(crate) global_vars: Vec<Value>,
    pub(crate) tables: Tables,
    pub(crate) memory: BumpProxy,
}

pub struct CallFrame {
    /// Store return addresses of Lane calls
    pub instr_ptr: usize,
    /// beginning of the local stack
    pub stack_offset: usize,
}

impl RuntimeData {
    pub fn new(
        memory_capacity: usize,
        stack_size: usize,
        call_stack_size: usize,
    ) -> Result<Self, ExecutionError> {
        let memory: BumpProxy = BumpAllocator::new(memory_capacity).into();
        let res = Self {
            stack: ValueStack::new(stack_size),
            call_stack: BoundedStack::new(call_stack_size),
            tables: KeyMap::with_capacity(128, memory.clone())
                .map_err(|_| ExecutionError::OutOfMemory)?,
            global_vars: Vec::with_capacity(16),
            memory,
        };
        Ok(res)
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.global_vars.clear();
        self.call_stack.clear();
        unsafe {
            self.memory.get_inner().clear();
        }
    }

    /// implies clear
    pub fn set_memory_limit(&mut self, capacity: usize) {
        self.clear();
        unsafe {
            self.memory.get_inner().reset(capacity);
        }
    }

    /// Types implementing Drop are not supported, thus the `Copy` bound
    pub fn write_to_memory<T: Sized + Copy>(&mut self, val: T) -> Result<Pointer, ExecutionError> {
        let l = std::alloc::Layout::new::<T>();
        unsafe {
            let ptr = self
                .memory
                .alloc(l)
                .map_err(|_| ExecutionError::OutOfMemory)?;

            std::ptr::write(ptr.as_ptr() as *mut T, val);
            Ok(Pointer(ptr.as_ptr()))
        }
    }
}
