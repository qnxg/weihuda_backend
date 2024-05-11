#![allow(non_snake_case)]
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::utils::default::default_page;
use crate::utils::serde::deserialize_naive_datetime;

#[derive(Deserialize, Debug)]
pub struct GetFeedbackReq {
    #[serde(default = "default_page")]
    pub page: u32,
    pub stuId: String,
}

#[derive(Deserialize, Debug)]
pub struct AddFeedbackReq {
    pub stuId: String,
    pub desc: String,
    pub contact: Option<String>,
    pub imgUrl: Option<String>,
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub createTime: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct UpdateFeedbackReq {
    pub id: u32,
    pub status: i8,
}
