use super::*;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

impl<T: Serialize> Serialize for KeyMap<T> {
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

struct KeyMapVisitor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for KeyMapVisitor<T> {
    type Value = KeyMap<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct KeyMap")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut cap = map.size_hint().unwrap_or(128);
        if !cap.is_power_of_two() {
            cap = cap.next_power_of_two();
        }
        let mut res = KeyMap::with_capacity(cap, SysAllocator::default()).expect("oom");
        while let Some((k, v)) = map.next_entry()? {
            res.insert(k, v).expect("oom");
        }
        Ok(res)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for KeyMap<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyMapVisitor {
            _m: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::{CaoIr, Lane},
        prelude::CaoProgram,
    };

    use super::*;

    #[test]
    fn can_serialize_json() {
        let mut map = KeyMap::default();
        map.insert(Handle(123), 69).unwrap();

        let js = serde_json::to_string(&map).unwrap();

        let map2: KeyMap<i32> = serde_json::from_str(&js).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_bincode() {
        let mut map = KeyMap::default();
        map.insert(Handle(123), 69).unwrap();

        let payload = bincode::serialize(&map).unwrap();

        let map2: KeyMap<i32> = bincode::deserialize(&payload).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_cbor() {
        let mut map = KeyMap::default();
        map.insert(Handle(123), 69).unwrap();

        let mut payload = Vec::new();
        ciborium::ser::into_writer(&map, &mut payload).unwrap();

        let map2: KeyMap<i32> = ciborium::de::from_reader(&payload[..]).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_program_cbor() {
        let program = crate::compiler::compile(
            &CaoIr {
                lanes: vec![Lane {
                    name: Some("poggers".into()),
                    arguments: vec![],
                    cards: vec![],
                }],
            },
            None,
        )
        .unwrap();

        let mut payload = Vec::new();
        ciborium::ser::into_writer(&program, &mut payload).unwrap();

        let _: CaoProgram = ciborium::de::from_reader(payload.as_slice()).unwrap();
    }
}
