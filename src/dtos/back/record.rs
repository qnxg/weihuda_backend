#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetRecordReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    pub key: Option<String>,
    pub param: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GetRecordGoodsReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GetRecordRulesReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    pub key: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PostRecordReq {
    pub key: String,
    pub param: String,
}

#[derive(Deserialize, Debug)]
pub struct GetWebviewReq {
    pub url: String,
}
