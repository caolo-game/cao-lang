use std::hash::Hash;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{de, Deserialize, Deserializer};

// from https://github.com/serde-rs/json/issues/560#issuecomment-532054058
pub fn de_int_key<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Eq + Hash + FromStr,
    K::Err: Display,
    V: Deserialize<'de>,
{
    let string_map = <HashMap<String, V>>::deserialize(deserializer)?;
    let mut map = HashMap::with_capacity(string_map.len());
    for (s, v) in string_map {
        let k = K::from_str(&s).map_err(de::Error::custom)?;
        map.insert(k, v);
    }
    Ok(map)
}
