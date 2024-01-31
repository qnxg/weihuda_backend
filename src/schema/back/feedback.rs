#![allow(non_snake_case)]
use crate::utility::default::default_page;
use serde::Deserialize;

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
    pub createTime: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateFeedbackReq {
    pub id: u32,
    pub status: i8,
}
