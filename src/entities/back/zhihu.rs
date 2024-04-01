#![allow(non_snake_case)]
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::utils::serde::deserialize_option_naive_datetime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ZhihuListItem {
    pub id: Option<i32>,
    pub title: String,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub cover: Option<String>,
    pub status: Option<i32>,
    #[serde(deserialize_with = "deserialize_option_naive_datetime")]
    pub publishTime: Option<NaiveDateTime>,
    pub stuId: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZhihuPage {
    pub count: u32,
    pub rows: Vec<ZhihuListItem>,
}
