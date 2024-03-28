use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::utility::default::default_page;

/// 获取月流量明细
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GetZhihuPageReq {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub pageSize: u32,
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
    pub publishTime: Option<NaiveDateTime>,
    pub stuId: Option<String>,
}

fn default_page_size() -> u32 {
    10
}
