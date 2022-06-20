use std::{alloc::Layout, ptr::NonNull, str::FromStr};

use crate::{
    alloc::{Allocator, BumpAllocator, BumpProxy},
    collections::{
        bounded_stack::BoundedStack,
        key_map::{KeyMap, MapError},
        value_stack::ValueStack,
    },
    prelude::*,
    value::Value,
};
use tracing::debug;

pub struct FieldTable {
    keys: KeyMap<Value, BumpProxy>,
    values: KeyMap<Value, BumpProxy>,
    alloc: BumpProxy,
}

impl Clone for FieldTable {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter(), self.alloc.clone()).unwrap()
    }
}

impl std::fmt::Debug for FieldTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.keys
                    .iter()
                    .zip(self.values.iter())
                    .map(|((h1, k), (h2, v))| {
                        debug_assert!(h1 == h2);
                        (k, v)
                    }),
            )
            .finish()
    }
}

impl FieldTable {
    pub fn with_capacity(size: usize, proxy: BumpProxy) -> Result<Self, MapError> {
        let res = Self {
            keys: KeyMap::with_capacity(size, proxy.clone())?,
            values: KeyMap::with_capacity(size, proxy.clone())?,
            alloc: proxy,
        };
        Ok(res)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    // keys can not be mutated
    pub fn get_key(&self, handle: Handle) -> Option<Value> {
        self.keys.get(handle).copied()
    }

    pub fn get_value(&self, handle: Handle) -> Option<Value> {
        self.values.get(handle).copied()
    }
    pub fn get_value_mut(&mut self, handle: Handle) -> Option<&mut Value> {
        self.values.get_mut(handle)
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
        let handle = Self::hash_value(key)?;
        self.keys
            .insert(handle, key)
            .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;
        self.values
            .insert(handle, value)
            .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;

        Ok(())
    }

    fn hash_value(key: Value) -> Result<Handle, ExecutionErrorPayload> {
        let handle = match key {
            Value::Nil => Handle::default(),
            Value::String(s) => {
                let s = unsafe {
                    s.get_str().ok_or_else(|| {
                        ExecutionErrorPayload::invalid_argument("String not found".to_string())
                    })?
                };
                Handle::from_str(s).unwrap()
            }
            Value::Integer(i) => Handle::from(i),
            Value::Real(_) | Value::Object(_) => return Err(ExecutionErrorPayload::Unhashable),
        };
        Ok(handle)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Value, Value)> + '_ {
        self.keys
            .iter()
            .zip(self.values.iter())
            .map(|((k1, k), (k2, v))| {
                debug_assert!(k1 == k2);
                (*k, *v)
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Value, &mut Value)> + '_ {
        self.keys
            .iter()
            .zip(self.values.iter_mut())
            .map(|((k1, k), (k2, v))| {
                debug_assert!(k1 == k2);
                (*k, v)
            })
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
