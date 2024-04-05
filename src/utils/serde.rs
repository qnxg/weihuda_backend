use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer};

pub fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
}

pub fn deserialize_option_naive_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;

    match s {
        Some(s) => NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// 当数据整数时，序列化为整数，而不是显示类似4.0的浮点数
/// 使用方式：use super::serialize_f64;
pub fn serialize_f64<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if x.fract() == 0.0 {
        s.serialize_u64(*x as u64)
    } else {
        s.serialize_f64(*x)
    }
}
