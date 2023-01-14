use crate::{
    alloc::BumpProxy,
    collections::hash_map::{CaoHashMap, MapError},
    prelude::*,
    value::Value,
};

pub struct CaoLangTable {
    map: CaoHashMap<Value, Value, BumpProxy>,
    keys: Vec<Value>,
    alloc: BumpProxy,
}

impl Clone for CaoLangTable {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().map(|(k, v)| (*k, *v)), self.alloc.clone()).unwrap()
    }
}

impl std::fmt::Debug for CaoLangTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.map.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl CaoLangTable {
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
        self.keys.retain(|k| {
            let remove = k == &key;
            if remove {
                self.map.remove(k);
            }
            remove
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
        self.keys
            .iter()
            .filter_map(|k| self.map.get(k).map(|v| (k, v)))
    }
}

impl std::ops::Deref for CaoLangTable {
    type Target = CaoHashMap<Value, Value, BumpProxy>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
