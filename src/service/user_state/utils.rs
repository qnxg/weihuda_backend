use std::{collections::HashMap, str::FromStr};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize, ser::SerializeMap};

pub struct SerializableHeaderMap(HeaderMap);

impl SerializableHeaderMap {
    pub fn new(header_map: HeaderMap) -> Self {
        Self(header_map)
    }
    pub fn into_inner(self) -> HeaderMap {
        self.0
    }
}

impl Serialize for SerializableHeaderMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0.iter() {
            let key = key.as_str();
            let value = value.to_str().map_err(|e| {
                serde::ser::Error::custom(format!("{:?}", e))
            })?;
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for SerializableHeaderMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, String> =
            HashMap::deserialize(deserializer)?;
        let mut header_map = HeaderMap::new();
        for (key, value) in map {
            let key = HeaderName::from_str(&key).map_err(|e| {
                serde::de::Error::custom(format!("{:?}", e))
            })?;
            let value =
                HeaderValue::from_str(&value).map_err(|e| {
                    serde::de::Error::custom(format!("{:?}", e))
                })?;
            header_map.insert(key, value);
        }
        Ok(SerializableHeaderMap(header_map))
    }
}
