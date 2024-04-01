#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

///一卡通信息，爬虫解析和请求返回共用
#[derive(Deserialize, Debug)]
pub struct SpiderCardInfo {
    pub account: u32,
    pub balance: String,
}

#[derive(Serialize, Debug)]
pub struct CardInfoRes {
    pub account: u32,
    pub balance: f64,
}

/// 一卡通历史账单
#[derive(Deserialize, Debug)]
pub struct SpiderCardHistory {
    pub TranCount: f64,
    pub total: f64,
    pub items: Vec<SpiderCardHistoryItem>,
}

#[derive(Deserialize, Debug)]
pub struct SpiderCardHistoryItem {
    pub fTranAmt: String,
    pub effectdate: String,
    pub jndatetime: String,
    pub jourName: String,
    pub usedcardnum: u32,
    pub nowAmt: String,
    pub sysname1: Option<String>,
    pub tranname: String,
}

#[derive(Serialize, Debug)]
pub struct CardHistoryRes {
    pub TranCount: f64,
    pub total: f64,
    pub items: Vec<CardHistoryResItem>,
}

#[derive(Serialize, Debug)]
pub struct CardHistoryResItem {
    pub tranAmt: String,
    pub effectDate: String,
    pub jourDate: String,
    pub jourName: String,
    pub jourNum: u32,
    pub nowAmt: String,
    pub tranLocation: String,
    pub tranname: String,
}
