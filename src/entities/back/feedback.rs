#![allow(non_snake_case)]
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug)]
pub struct FeedbackInfo {
    // pub id: u32,
    // pub contact: String,
    // pub createTime: String,
    pub desc: String,
    // pub imgUrl: String,
    pub stuId: Option<String>,
    // #[serde(rename = "type")]
    // pub _type: String,
    pub status: Option<i8>,
    pub comment: Option<String>,
}

#[derive(FromRow, Serialize, Debug)]
pub struct FeedbackRes {
    pub count: u32,
    pub rows: Vec<FeedbackInfo>,
}
