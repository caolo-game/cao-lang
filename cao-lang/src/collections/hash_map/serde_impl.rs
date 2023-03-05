use super::*;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

impl<K: Serialize, V: Serialize> Serialize for CaoHashMap<K, V> {
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

struct HashMapVisitor<K, V> {
    _m: std::marker::PhantomData<(K, V)>,
}

impl<'de, K: Deserialize<'de> + Eq + Hash, V: Deserialize<'de>> Visitor<'de>
    for HashMapVisitor<K, V>
{
    type Value = CaoHashMap<K, V>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct CaoHashMap")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut cap = map.size_hint().unwrap_or(128);
        if !cap.is_power_of_two() {
            cap = cap.next_power_of_two();
        }
        let mut res = CaoHashMap::with_capacity_in(cap, SysAllocator::default()).expect("oom");
        while let Some((k, v)) = map.next_entry()? {
            res.insert(k, v).expect("oom");
        }
        Ok(res)
    }
}

impl<'de, K: Deserialize<'de> + Hash + Eq, V: Deserialize<'de>> Deserialize<'de>
    for CaoHashMap<K, V>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(HashMapVisitor {
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
        let mut map = CaoHashMap::default();
        map.insert(123, 69).unwrap();

        let js = serde_json::to_string(&map).unwrap();

        let map2: CaoHashMap<i32, i32> = serde_json::from_str(&js).unwrap();

        let res = map2.get(&123).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_bincode() {
        let mut map = CaoHashMap::default();
        map.insert(123, "poggers".to_string()).unwrap();
        map.insert(42, "coggers".to_string()).unwrap();

        let payload = bincode::serialize(&map).unwrap();

        let map2: CaoHashMap<i32, String> = bincode::deserialize(&payload).unwrap();

        let res = map2.get(&123).unwrap();
        assert_eq!(*res, "poggers");
        let res = map2.get(&42).unwrap();
        assert_eq!(*res, "coggers");
    }

    #[test]
    fn can_serialize_cbor() {
        let mut map = CaoHashMap::default();
        map.insert(123, 69).unwrap();

        let mut payload = Vec::new();
        ciborium::ser::into_writer(&map, &mut payload).unwrap();

        let map2: CaoHashMap<i32, i32> = ciborium::de::from_reader(&payload[..]).unwrap();

        let res = map2.get(&123).unwrap();
        assert_eq!(*res, 69);
    }

    #[test]
    fn can_serialize_program_cbor() {
        let program = crate::compiler::compile(
            CaoProgram {
                imports: Default::default(),
                lanes: [(
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
