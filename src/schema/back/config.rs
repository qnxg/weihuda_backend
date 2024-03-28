#![allow(non_snake_case)]

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetConfigReq {
    pub page: Option<u32>,
    pub pageSize: Option<u32>,
    pub key: Option<String>,
}
