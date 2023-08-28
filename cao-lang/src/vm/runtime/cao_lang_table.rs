use crate::{
    alloc::AllocProxy,
    collections::hash_map::{CaoHashMap, MapError},
    prelude::*,
    value::Value,
};

pub struct CaoLangTable {
    map: CaoHashMap<Value, Value, AllocProxy>,
    keys: Vec<Value>,
    alloc: AllocProxy,
}

impl Clone for CaoLangTable {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().map(|(k, v)| (*k, *v)), self.alloc.clone()).unwrap()
    }
}

impl std::fmt::Debug for CaoLangTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.keys.iter().map(|k| (k, self.map.get(k))))
            .finish()
    }
}

impl CaoLangTable {
    pub fn with_capacity(size: usize, proxy: AllocProxy) -> Result<Self, MapError> {
        let res = Self {
            map: CaoHashMap::with_capacity_in(size, proxy.clone())?,
            keys: Vec::default(),
            alloc: proxy,
        };
        Ok(res)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn from_iter(
        it: impl Iterator<Item = (Value, Value)>,
        alloc: AllocProxy,
    ) -> Result<Self, ExecutionErrorPayload> {
        let mut result = Self::with_capacity(it.size_hint().0, alloc)
            .map_err(|_err| ExecutionErrorPayload::OutOfMemory)?;

        for (key, value) in it {
            result.insert(key, value)?;
        }

        Ok(result)
    }

    pub fn insert(
        &mut self,
        key: impl Into<Value>,
        value: impl Into<Value>,
    ) -> Result<(), ExecutionErrorPayload> {
        fn _insert(
            this: &mut CaoLangTable,
            key: Value,
            value: Value,
        ) -> Result<(), ExecutionErrorPayload> {
            match this.map.get_mut(&key) {
                Some(r) => {
                    *r = value;
                }
                None => {
                    this.map
                        .insert(key, value)
                        .map_err(|_| ExecutionErrorPayload::OutOfMemory)?;
                    this.keys.push(key);
                }
            }

            Ok(())
        }

        _insert(self, key.into(), value.into())
    }

    pub fn remove(&mut self, key: Value) -> Result<(), ExecutionErrorPayload> {
        self.keys.retain(|k| {
            let retain = k != &key;
            if !retain {
                self.map.remove(k);
            }
            retain
        });
        Ok(())
    }

    pub fn append(&mut self, value: Value) -> Result<(), ExecutionErrorPayload> {
        let mut index = self.keys.len() as i64;
        while self.map.contains(&Value::Integer(index)) {
            index += 1;
        }
        self.insert(Value::Integer(index), value)
    }

    pub fn pop(&mut self) -> Result<Value, ExecutionErrorPayload> {
        match self.keys.pop() {
            Some(key) => {
                let res = self.get(&key).copied().unwrap_or(Value::Nil);
                self.remove(key)?;
                Ok(res)
            }
            None => return Ok(Value::Nil),
        }
    }

    pub fn nth_key(&self, i: usize) -> Value {
        if i >= self.keys.len() {
            return Value::Nil;
        }
        self.keys[i]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> + '_ {
        self.keys
            .iter()
            .filter_map(|k| self.map.get(k).map(|v| (k, v)))
    }

    pub fn keys(&self) -> &[Value] {
        &self.keys
    }

    pub fn keys_mut(&mut self) -> &mut [Value] {
        &mut self.keys
    }
}

impl std::ops::Deref for CaoLangTable {
    type Target = CaoHashMap<Value, Value, AllocProxy>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
