use super::*;
use ::serde::{de::SeqAccess, de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

impl<T: Serialize> Serialize for KeyMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let len = self.len();
        let mut count = 0;
        let mut seq = serializer.serialize_seq(Some(len))?;
        for (key, val) in self.iter() {
            seq.serialize_element(&(key.0, val))?;
            count += 1;
        }
        debug_assert_eq!(count, len);
        seq.end()
    }
}

struct PhmVisitor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for PhmVisitor<T> {
    type Value = KeyMap<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A list of nodeid-label tuples")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut res = KeyMap::<T>::default();
        while let Some((k, v)) = seq.next_element()? {
            res.insert(Handle(k), v).expect("oom");
        }
        Ok(res)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for KeyMap<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(PhmVisitor {
            _m: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_serialize() {
        let mut map = KeyMap::default();
        map.insert(Handle(123), 69).unwrap();

        let js = serde_json::to_string(&map).unwrap();

        let map2: KeyMap<i32> = serde_json::from_str(&js).unwrap();

        let res = map2.get(Handle(123)).unwrap();
        assert_eq!(*res, 69);
    }
}
