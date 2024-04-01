use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::utils::serde::deserialize_option_naive_datetime;

/// 获取月流量明细
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GetZhihuPageReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CrudZhihuByIdReq {
    pub id: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct UpdateZhihuBody {
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
