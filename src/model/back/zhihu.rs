#![allow(non_snake_case)]
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utility::parse::deserialize_option_naive_datetime;

#[derive(FromRow, Serialize, Deserialize, Debug)]
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

// impl Default for ZhihuPage {
//     fn default() -> Self {
//         Self {
//             count: 0,
//             content: vec![ZhihuListItem {
//                 id: String::default(),
//                 typetag: String::default(),
//                 title: "知湖暂时下线，待完善后再开放".to_string(),
//                 address: String::default(),
//                 date_begin: String::default(),
//                 is_top: String::default(),
//                 editor: String::default(),
//                 create_time: "2024-2-26".to_string(),
//                 image: String::default(),
//             }],
//         }
//     }
// }
