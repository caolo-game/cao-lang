use super::*;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

impl<T: Serialize> Serialize for HandleTable<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_map(self.len().into())?;
        for (k, v) in self.iter() {
            state.serialize_entry(&k, v)?;
        }
        state.end()
    }
}

struct HandleTableVisitor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for HandleTableVisitor<T> {
    type Value = HandleTable<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct HandleTable")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut cap = map.size_hint().unwrap_or(128);
        if !cap.is_power_of_two() {
            cap = cap.next_power_of_two();
        }
        let mut res = HandleTable::with_capacity(cap, SysAllocator::default()).expect("oom");
        while let Some((k, v)) = map.next_entry()? {
            res.insert(k, v).expect("oom");
        }
        Ok(res)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for HandleTable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(HandleTableVisitor {
            _m: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::{CaoProgram, Function},
        prelude::CaoCompiledProgram,
    };

    use super::*;

    #[test]
    fn can_serialize_json() {
        let mut map = HandleTable::default();
        map.insert(Handle(123), 69).unwrap();

        let js = serde_json::to_string(&map).unwrap();

        let map2: HandleTable<i32> = serde_json::from_str(&js).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_bincode() {
        let mut map = HandleTable::default();
        map.insert(Handle(123), "poggers".to_string()).unwrap();
        map.insert(Handle(42), "coggers".to_string()).unwrap();

        let payload = bincode::serialize(&map).unwrap();

        let map2: HandleTable<String> = bincode::deserialize(&payload).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, "poggers");
        let res = map2.get(Handle(42)).unwrap();
        assert_eq!(*res, "coggers");
    }

    #[test]
    fn can_serialize_cbor() {
        let mut map = HandleTable::default();
        map.insert(Handle(123), 69).unwrap();

        let mut payload = Vec::new();
        ciborium::ser::into_writer(&map, &mut payload).unwrap();

        let map2: HandleTable<i32> = ciborium::de::from_reader(&payload[..]).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_program_cbor() {
        let program = crate::compiler::compile(
            CaoProgram {
                imports: Default::default(),
                functions: [(
                    "main".into(),
                    Function {
                        arguments: vec![],
                        cards: vec![],
                    },
                )]
                .into(),
                submodules: Default::default(),
            },
            None,
        )
        .unwrap();

        let mut payload = Vec::new();
        ciborium::ser::into_writer(&program, &mut payload).unwrap();

        let _: CaoCompiledProgram = ciborium::de::from_reader(payload.as_slice()).unwrap();
    }
}
