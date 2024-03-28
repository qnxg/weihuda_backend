#![allow(non_snake_case)]
use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug)]
pub struct MiniBindRecord {
    pub jifen: Option<u32>,
    pub stuID: Option<String>,
    pub id: u32,
}

#[derive(FromRow, Serialize, Debug)]
pub struct Record {
    pub id: u32,
    pub key: String,
    pub param: String,
    pub stuId: String,
    pub jifen: i32,
    pub desc: String,
    pub createTime: NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct RecordRes {
    pub count: u32,
    pub rows: Vec<Record>,
}

#[derive(FromRow, Serialize, Debug)]
pub struct RecordGoods {
    pub id: u32,
    pub name: String,
    pub cover: String,
    pub count: i32,
    pub price: i32,
    pub description: Option<String>,
    pub enabled: Option<i8>,
}

#[derive(Serialize, Debug)]
pub struct RecordGoodsRes {
    pub count: u32,
    pub rows: Vec<RecordGoods>,
}

#[derive(FromRow, Serialize, Debug)]
pub struct RecordRules {
    pub id: u32,
    pub key: String,
    pub name: String,
    pub jifen: i32,
    pub cycle: i32,
    pub maxCount: i32,
}

#[derive(Serialize, Debug)]
pub struct RecordRulesRes {
    pub count: u32,
    pub rows: Vec<RecordRules>,
}

#[derive(Serialize, Debug)]
pub struct PostRecordRes {
    pub jifen: i32,
}

/// 兑换的商品
#[derive(Deserialize, Debug)]
pub struct Goods {
    pub id: u32,
    pub stuId: String,
    pub goodsId: i32,
    pub exchangeTime: NaiveDateTime,
    pub status: Option<i32>,
    pub receiveTime: Option<NaiveDateTime>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GoodsReq {
    pub goodsId: i32,
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub exchangeTime: NaiveDateTime,
}

// #[derive(FromRow,Serialize, Debug)]
// pub struct RecordGoodsList {
//     pub id: u32,
//     pub name: String,
//     pub cover: String,
//     pub count: i32,
//     pub price: i32,
//     pub description: Option<String>,
//     pub createTime: NaiveDateTime,
// }

fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
}
