#![allow(non_snake_case)]
use serde::Deserialize;

/// 获取月流量明细
#[derive(Deserialize, Debug)]
pub struct GetNetflowMonthDetailReq {
    pub year: String,
    pub month: String,
}

/// 获取日流量明细
#[derive(Deserialize, Debug)]
pub struct GetNetflowDayDetailReq {
    pub year: String,
    pub month: String,
    pub day: String,
}
