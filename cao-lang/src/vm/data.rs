use crate::alloc::BumpAllocator;
use crate::collections::{bounded_stack::BoundedStack, value_stack::ValueStack};
use crate::{prelude::*, value::Value};

pub struct RuntimeData {
    pub stack: ValueStack,
    pub call_stack: BoundedStack<CallFrame>,
    pub global_vars: Vec<Value>,
    pub memory: BumpAllocator,
}

pub struct CallFrame {
    /// Store return addresses of Lane calls
    pub instr_ptr: usize,
    /// beginning of the local stack
    pub stack_offset: usize,
}

impl RuntimeData {
    pub fn clear(&mut self) {
        self.stack.clear();
        self.global_vars.clear();
        self.call_stack.clear();
        unsafe {
            self.memory.clear();
        }
    }

    pub fn set_memory_limit(&mut self, capacity: usize) {
        unsafe {
            self.memory.reset(capacity);
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
