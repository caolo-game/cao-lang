use std::{alloc::Layout, ptr::NonNull};

use crate::{
    alloc::{Allocator, BumpAllocator, BumpProxy},
    collections::{
        bounded_stack::BoundedStack,
        hash_map::{CaoHashMap, MapError},
        value_stack::ValueStack,
    },
    prelude::*,
    value::Value,
};
use tracing::debug;

pub struct FieldTable {
    map: CaoHashMap<Value, Value, BumpProxy>,
    keys: Vec<Value>,
    alloc: BumpProxy,
}

impl Clone for FieldTable {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().map(|(k, v)| (*k, *v)), self.alloc.clone()).unwrap()
    }
}

impl std::fmt::Debug for FieldTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl FieldTable {
    pub fn with_capacity(size: usize, proxy: BumpProxy) -> Result<Self, MapError> {
        let res = Self {
            map: CaoHashMap::with_capacity_in(size, proxy.clone())?,
            keys: Vec::default(),
            alloc: proxy,
        };
        Ok(res)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn from_iter(
        it: impl Iterator<Item = (Value, Value)>,
        alloc: BumpProxy,
    ) -> Result<Self, ExecutionErrorPayload> {
        let mut result = Self::with_capacity(it.size_hint().0, alloc)
            .map_err(|_err| ExecutionErrorPayload::OutOfMemory)?;

        for (key, value) in it {
            result.insert(key, value)?;
        }

        Ok(result)
    }

    pub fn insert(&mut self, key: Value, value: Value) -> Result<(), ExecutionErrorPayload> {
        self.map
            .insert(key, value)
            .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;
        self.keys.push(key);

        Ok(())
    }

    pub fn remove(&mut self, key: Value) -> Result<(), ExecutionErrorPayload> {
        self.map.remove(&key);
        // key is not removed from `keys`
        // TODO: gc should rebuild the keys?
        Ok(())
    }

    pub fn rebuild_keys(&mut self) {
        self.keys.clear();
        self.keys.extend(self.map.iter().map(|(k, _)| *k));
    }

    pub fn nth_key(&self, i: usize) -> Value {
        if i >= self.keys.len() {
            return Value::Nil;
        }
        self.keys[i]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> + '_ {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Value, &mut Value)> + '_ {
        self.map.iter_mut()
    }
}

impl std::ops::Deref for FieldTable {
    type Target = CaoHashMap<Value, Value, BumpProxy>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

pub struct RuntimeData {
    pub(crate) value_stack: ValueStack,
    pub(crate) call_stack: BoundedStack<CallFrame>,
    pub(crate) global_vars: Vec<Value>,
    pub(crate) memory: BumpProxy,
    pub(crate) object_list: Vec<NonNull<FieldTable>>,
}

impl Drop for RuntimeData {
    fn drop(&mut self) {
        self.clear();
    }
}

pub(crate) struct CallFrame {
    /// Store src addresses of Lane calls
    pub src_instr_ptr: u32,
    /// Store return addresses of Lane calls
    pub dst_instr_ptr: u32,
    /// beginning of the local stack
    pub stack_offset: u32,
}

impl RuntimeData {
    pub fn new(
        memory_capacity: usize,
        stack_size: usize,
        call_stack_size: usize,
    ) -> Result<Self, ExecutionErrorPayload> {
        let memory: BumpProxy = BumpAllocator::new(memory_capacity).into();
        let res = Self {
            value_stack: ValueStack::new(stack_size),
            call_stack: BoundedStack::new(call_stack_size),
            global_vars: Vec::with_capacity(16),
            object_list: Vec::with_capacity(16),
            memory,
        };
        Ok(res)
    }

    /// Initialize a new cao-lang table and return a pointer to it
    pub fn init_table(&mut self) -> Result<NonNull<FieldTable>, ExecutionErrorPayload> {
        unsafe {
            let table_ptr = self
                .memory
                .alloc(Layout::new::<FieldTable>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;
            let table = FieldTable::with_capacity(8, self.memory.clone()).map_err(|err| {
                debug!("Failed to init table {:?}", err);
                ExecutionErrorPayload::OutOfMemory
            })?;

            let table_ptr: NonNull<FieldTable> = table_ptr.cast();
            std::ptr::write(table_ptr.as_ptr(), table);
            self.object_list.push(table_ptr);

            Ok(table_ptr)
        }
    }

    pub fn clear(&mut self) {
        self.clear_objects();
        self.value_stack.clear();
        self.global_vars.clear();
        self.call_stack.clear();
        unsafe {
            self.memory.get_inner().clear();
        }
    }

    fn clear_objects(&mut self) {
        for obj in self.object_list.iter_mut() {
            unsafe {
                std::ptr::drop_in_place(obj.as_ptr());
            }
        }
        self.object_list.clear();
    }

    /// implies clear
    pub fn set_memory_limit(&mut self, capacity: usize) {
        self.clear();
        unsafe {
            self.memory.get_inner().reset(capacity);
        }
    }

    /// Types implementing Drop are not supported, thus the `Copy` bound
    pub fn write_to_memory<T: Sized + Copy>(
        &mut self,
        val: T,
    ) -> Result<*mut T, ExecutionErrorPayload> {
        let l = std::alloc::Layout::new::<T>();
        unsafe {
            let ptr = self
                .memory
                .alloc(l)
                .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;

            std::ptr::write(ptr.as_ptr() as *mut T, val);
            Ok(ptr.as_ptr() as *mut T)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_table_can_be_queried_by_str_test() {
        let mut vm = Vm::new(()).unwrap();

        let s = vm.init_string("poggers").unwrap();
        let o = unsafe { vm.init_table().unwrap().as_mut() };

        o.insert(Value::String(s), Value::Integer(42)).unwrap();

        let res = o.get("poggers").unwrap();

        assert_eq!(res, &Value::Integer(42));
    }
}
