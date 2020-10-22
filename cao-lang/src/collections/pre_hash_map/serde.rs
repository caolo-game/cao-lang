use super::*;
use ::serde::{de::SeqAccess, de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

impl<T: Serialize> Serialize for PreHashMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for kv in self.iter() {
            seq.serialize_element(&kv)?;
        }
        seq.end()
    }
}

struct PHMVisitor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for PHMVisitor<T> {
    type Value = PreHashMap<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A list of nodeid-label tuples")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut res = PreHashMap::<T>::default();
        while let Some((k, v)) = seq.next_element()? {
            res.insert(k, v);
        }
        Ok(res)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for PreHashMap<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(PHMVisitor {
            _m: Default::default(),
        })
    }
}
