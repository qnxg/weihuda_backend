#![allow(dead_code)]
use crate::{app_result::AppResult, extractors::Json};

#[derive(serde::Deserialize, Debug)]
pub struct TestNaiveDateTimeParsingReq {
    #[serde(deserialize_with = "crate::utils::serde::deserialize_naive_datetime")]
    pub time: chrono::NaiveDateTime,
}

pub async fn test_naive_datetime_parsing(
    Json(json): Json<TestNaiveDateTimeParsingReq>,
) -> AppResult {
    dbg!(json);
    Ok(().into())
}

#[derive(serde::Deserialize, Debug)]
pub struct TestOptionNaiveDateTimeParsingReq {
    #[serde(default)]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_option_naive_datetime")]
    pub time: Option<chrono::NaiveDateTime>,
}

pub async fn test_option_naive_datetime_parsing(
    Json(json): Json<TestOptionNaiveDateTimeParsingReq>,
) -> AppResult {
    dbg!(json);
    Ok(().into())
}
