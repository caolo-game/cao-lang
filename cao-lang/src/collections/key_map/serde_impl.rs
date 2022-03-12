use super::*;
use ::serde::{
    de::Visitor,
    de::{self, SeqAccess},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

impl<T: Serialize> Serialize for KeyMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("KeyMap", 1)?;
        let values = self.iter().collect::<Vec<_>>();
        state.serialize_field("values", &values)?;
        state.end()
    }
}

struct PhmVisitor<T> {
    _m: std::marker::PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for PhmVisitor<T> {
    type Value = KeyMap<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct KeyMap")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut res = KeyMap::<T>::default();
        let values: Vec<(Handle, T)> = seq
            .next_element()?
            .ok_or_else(|| de::Error::missing_field("values"))?;
        for (k, v) in values {
            res.insert(k, v).expect("oom");
        }
        Ok(res)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut values = None;
        while let Some(key) = map.next_key()? {
            match key {
                "values" => {
                    values = map.next_value()?;
                }
                _ => {}
            }
        }
        let values: Vec<(Handle, T)> = values.ok_or_else(|| de::Error::missing_field("values"))?;

        let mut res = KeyMap::default();
        for (k, v) in values {
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
        deserializer.deserialize_struct(
            "KeyMap",
            &["values"],
            PhmVisitor {
                _m: Default::default(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
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
}
