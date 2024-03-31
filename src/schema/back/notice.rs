#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetNoticeReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    // #[serde(deserialize_with = "deserialize_option_naive_datetime")]
    // pub sendTime: Option<NaiveDateTime>,
    pub status: Option<i32>,
    pub result: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PutNoticeByIdReq {
    pub result: Option<String>,
    pub status: Option<i32>,
}
