use std::str::FromStr;

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_naive_datetime<'de, D>(
    deserializer: D,
) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt - Duration::hours(8))
        .map_err(serde::de::Error::custom)
}

pub fn deserialize_option_naive_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;

    match s {
        Some(s) => {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt - Duration::hours(8))
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        None => Ok(None),
    }
}

pub fn empty_string_as_none<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> Deserialize<'a> + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    // 先反序列化为 Value，不消费 deserializer
    let value = Value::deserialize(deserializer)?;

    match value {
        Value::String(s) if s.is_empty() => Ok(None),
        Value::String(s) => {
            s.parse().map(Some).map_err(serde::de::Error::custom)
        }
        Value::Null => Ok(None),
        _ => {
            // 对于其他类型，使用 serde_json 重新反序列化
            serde_json::from_value(value)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}
