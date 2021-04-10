use crate::collections::{bounded_stack::BoundedStack, scalar_stack::ScalarStack};
use crate::{prelude::*, scalar::Scalar};

pub struct RuntimeData {
    pub memory_limit: usize,

    pub stack: ScalarStack,
    pub call_stack: BoundedStack<CallFrame>,
    pub memory: Vec<u8>,
    pub global_vars: Vec<Scalar>,
}

pub struct CallFrame {
    /// Store return addresses of Lane calls
    pub instr_ptr: usize,
}

impl RuntimeData {
    pub fn clear(&mut self) {
        self.memory.clear();
        self.stack.clear();
        self.global_vars.clear();
        self.call_stack.clear();
    }

    pub fn write_to_memory<T: ByteEncodeProperties>(
        &mut self,
        val: T,
    ) -> Result<(Pointer, usize), ExecutionError> {
        let result = self.memory.len();

        val.encode(&mut self.memory).map_err(|err| {
            ExecutionError::invalid_argument(format!("Failed to encode argument {:?}", err))
        })?;

        if self.memory.len() >= self.memory_limit {
            return Err(ExecutionError::OutOfMemory);
        }
        Ok((Pointer(result as u32), self.memory.len() - result))
    }

    pub fn get_value_in_place<'a, T: DecodeInPlace<'a>>(
        &'a self,
        object: &Object,
    ) -> Option<<T as DecodeInPlace<'a>>::Ref> {
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
            None => None,
        }
    }

    pub fn get_value<T: ByteDecodeProperties>(&self, object: &Object) -> Option<T> {
        match object.index {
            Some(index) => {
                let data = &self.memory;
                let head = index.0 as usize;
                let tail = (head.checked_add(object.size as usize))
                    .unwrap_or(data.len())
                    .min(data.len());
                T::decode(&data[head..tail]).ok().map(|(_, val)| val)
            }
            None => None,
        }
    }
}
