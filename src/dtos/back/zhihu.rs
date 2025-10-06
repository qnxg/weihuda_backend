use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::utils::serde::deserialize_option_naive_datetime;

/// 获取知湖页列表
#[derive(Deserialize, Debug)]
pub struct GetZhihuPageReq {
    pub offset: u32,
    pub req_count: u32,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CrudZhihuByIdReq {
    pub id: u32,
}

#[expect(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct UpdateZhihuBody {
    pub title: String,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub cover: Option<String>,
    pub status: Option<i32>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_naive_datetime")]
    pub publishTime: Option<NaiveDateTime>,
    pub stuId: Option<String>,
}
