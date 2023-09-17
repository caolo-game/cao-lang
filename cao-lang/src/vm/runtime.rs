pub mod cao_lang_function;
pub mod cao_lang_object;
pub mod cao_lang_string;
pub mod cao_lang_table;

use std::{alloc::Layout, pin::Pin, ptr::NonNull};

use crate::{
    alloc::{AllocProxy, Allocator, CaoLangAllocator},
    collections::{bounded_stack::BoundedStack, value_stack::ValueStack},
    prelude::*,
    value::Value,
    vm::runtime::cao_lang_object::CaoLangObjectBody,
};
use tracing::debug;

use self::{
    cao_lang_function::{CaoLangClosure, CaoLangFunction, CaoLangNativeFunction, CaoLangUpvalue},
    cao_lang_object::{CaoLangObject, GcMarker, ObjectGcGuard},
    cao_lang_string::CaoLangString,
};

pub struct RuntimeData {
    pub(crate) value_stack: ValueStack,
    pub(crate) call_stack: BoundedStack<CallFrame>,
    pub(crate) global_vars: Vec<Value>,
    pub(crate) memory: AllocProxy,
    pub(crate) object_list: Vec<NonNull<CaoLangObject>>,
    pub(crate) current_program: *const CaoCompiledProgram,
    pub(crate) open_upvalues: *mut CaoLangObject,
}

impl Drop for RuntimeData {
    fn drop(&mut self) {
        self.clear();
    }
}

pub(crate) struct CallFrame {
    /// Store src addresses of Function calls
    pub src_instr_ptr: u32,
    /// Store return addresses of Function calls
    pub dst_instr_ptr: u32,
    /// beginning of the local stack
    pub stack_offset: u32,
    pub closure: *mut CaoLangClosure,
}

impl RuntimeData {
    pub fn new(
        memory_limit: usize,
        stack_size: usize,
        call_stack_size: usize,
    ) -> Result<Pin<Box<Self>>, ExecutionErrorPayload> {
        // we have a chicken-egg problem if we want to store the allocator in this structure
        let allocator = CaoLangAllocator::new(std::ptr::null_mut(), memory_limit);
        let memory: AllocProxy = allocator.into();
        let mut res = Box::pin(Self {
            value_stack: ValueStack::new(stack_size),
            call_stack: BoundedStack::new(call_stack_size),
            global_vars: Vec::with_capacity(16),
            object_list: Vec::with_capacity(16),
            memory,
            current_program: std::ptr::null(),
            open_upvalues: std::ptr::null_mut(),
        });
        unsafe {
            let reference: &mut Self = Pin::get_mut(res.as_mut());
            res.memory.get_inner().runtime = reference as *mut Self;
        }
        Ok(res)
    }

    /// Initialize a new cao-lang table and return a pointer to it
    pub fn init_table(&mut self) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;
            let table = CaoLangTable::with_capacity(8, self.memory.clone()).map_err(|err| {
                debug!("Failed to init table {:?}", err);
                ExecutionErrorPayload::OutOfMemory
            })?;

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::Table(table),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);
            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn init_native_function(
        &mut self,
        handle: Handle,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate NativeFunction {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::NativeFunction(CaoLangNativeFunction { handle }),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);

            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn init_function(
        &mut self,
        handle: Handle,
        arity: u32,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::Function(CaoLangFunction { handle, arity }),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);

            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn init_closure(
        &mut self,
        handle: Handle,
        arity: u32,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::Closure(CaoLangClosure {
                    function: CaoLangFunction { handle, arity },
                    upvalues: vec![],
                }),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);

            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn init_upvalue(
        &mut self,
        location: *mut Value,
    ) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::Upvalue(CaoLangUpvalue {
                    location,
                    value: Value::Nil,
                    next: std::ptr::null_mut(),
                }),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);

            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn init_string(&mut self, payload: &str) -> Result<ObjectGcGuard, ExecutionErrorPayload> {
        unsafe {
            let obj_ptr = self
                .memory
                .alloc(Layout::new::<CaoLangObject>())
                .map_err(|err| {
                    debug!("Failed to allocate table {:?}", err);
                    ExecutionErrorPayload::OutOfMemory
                })?;

            let layout = CaoLangString::layout(payload.len());
            let mut ptr = self
                .memory
                .alloc(layout)
                .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;

            let result: *mut u8 = ptr.as_mut();
            std::ptr::copy(payload.as_ptr(), result, payload.len());

            let obj_ptr: NonNull<CaoLangObject> = obj_ptr.cast();
            let obj = CaoLangObject {
                marker: GcMarker::White,
                body: CaoLangObjectBody::String(CaoLangString {
                    len: payload.len(),
                    ptr,
                    alloc: self.memory.clone(),
                }),
            };
            std::ptr::write(obj_ptr.as_ptr(), obj);
            self.object_list.push(obj_ptr);

            Ok(ObjectGcGuard::new(obj_ptr))
        }
    }

    pub fn free_object(&mut self, obj: NonNull<CaoLangObject>) {
        unsafe {
            std::ptr::drop_in_place(obj.as_ptr());
            self.memory
                .dealloc(obj.cast(), Layout::new::<CaoLangObject>());
        }
    }

    pub fn clear(&mut self) {
        self.clear_objects();
        self.value_stack.clear();
        self.global_vars.clear();
        self.call_stack.clear();
    }

    fn clear_objects(&mut self) {
        for obj_ptr in std::mem::take(&mut self.object_list).into_iter() {
            self.free_object(obj_ptr);
        }
    }

    pub fn set_memory_limit(&mut self, capacity: usize) {
        self.clear();
        unsafe {
            self.memory
                .get_inner()
                .limit
                .store(capacity, std::sync::atomic::Ordering::Relaxed);
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

    pub fn gc(&mut self) {
        debug!("• GC");
        // mark all roots for collection
        let mut progress_tracker = Vec::with_capacity(self.value_stack.len());
        for val in self.value_stack.iter() {
            if let Value::Object(mut t) = val {
                unsafe {
                    let t = t.as_mut();
                    t.marker = GcMarker::Gray;
                    progress_tracker.push(t);
                }
            }
        }
        // mark globals
        for val in self.global_vars.iter() {
            if let Value::Object(mut t) = val {
                unsafe {
                    let t = t.as_mut();
                    t.marker = GcMarker::Gray;
                    progress_tracker.push(t);
                }
            }
        }

        macro_rules! checked_enqueue_value {
            ($val: ident) => {
                if let Value::Object(mut value) = $val {
                    let t = value.as_mut();
                    if matches!(t.marker, GcMarker::White) {
                        t.marker = GcMarker::Gray;
                        progress_tracker.push(t);
                    }
                }
            };
        }

        // mark referenced objects for collection
        while let Some(obj) = progress_tracker.pop() {
            obj.marker = GcMarker::Black;
            match &mut obj.body {
                CaoLangObjectBody::Table(obj) => {
                    for (key, value) in obj.iter() {
                        unsafe {
                            checked_enqueue_value!(key);
                            checked_enqueue_value!(value);
                        }
                    }
                }
                CaoLangObjectBody::Closure(c) => {
                    for upvalue in &mut c.upvalues {
                        unsafe {
                            let t = upvalue.as_mut();
                            if matches!(t.marker, GcMarker::White) {
                                t.marker = GcMarker::Gray;
                                progress_tracker.push(t);
                            }
                        }
                    }
                }
                CaoLangObjectBody::String(_) => {
                    // strings don't have children
                }
                CaoLangObjectBody::Function(_) => {
                    // function objects don't have children
                }
                CaoLangObjectBody::NativeFunction(_) => {
                    // native function objects don't have children
                }
                CaoLangObjectBody::Upvalue(u) => unsafe {
                    if let Some(t) = u.location.as_mut() {
                        checked_enqueue_value!(t);
                    }
                },
            }
        }
        // sweep
        //
        let mut collected = Vec::with_capacity(self.object_list.len());
        for (i, object) in self.object_list.iter().copied().enumerate() {
            unsafe {
                let obj = object.as_ref();
                if matches!(obj.marker, GcMarker::White) {
                    collected.push(i);
                }
            }
        }
        for i in collected.into_iter().rev() {
            let obj = self.object_list.swap_remove(i);
            self.free_object(obj);
        }
        // unmark remaning objects
        for table in self.object_list.iter_mut() {
            unsafe {
                let table = table.as_mut();
                if !matches!(table.marker, GcMarker::Protected) {
                    table.marker = GcMarker::White;
                }
            }
        }
        debug!("✓ GC");
    }

    pub fn capture_upvalue() {}
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use super::*;

    #[test]
    fn field_table_can_be_queried_by_str_test() {
        let mut vm = Vm::new(()).unwrap();

        let s = vm.init_string("poggers").unwrap();
        let mut o = vm.init_table().unwrap();
        let o = o.deref_mut().as_table_mut().unwrap();

        o.insert(Value::Object(s.into_inner()), Value::Integer(42))
            .unwrap();

        let res = o.get("poggers").unwrap();

        assert_eq!(res, &Value::Integer(42));
    }
}
