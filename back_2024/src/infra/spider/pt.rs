use serde::Deserialize;

use crate::{infra::spider::spider_data, result::AppResult};

#[derive(Deserialize, Debug)]
pub struct SpiderCardInfo {
    pub account: u32,
    pub balance: String,
}

pub async fn get_card_info(
    stu_id: &str,
) -> AppResult<SpiderCardInfo> {
    let params = [("stuid", stu_id)];
    let spider_res: SpiderCardInfo =
        spider_data("/pt/card/info", &params).await?;
    Ok(spider_res)
}

/// 一卡通历史账单
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderCardHistory {
    pub TranCount: f64,
    pub total: f64,
    pub items: Vec<SpiderCardHistoryItem>,
}
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
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
pub async fn get_card_history(
    stu_id: &str,
    year: &str,
    month: &str,
    typ: &str,
) -> AppResult<SpiderCardHistory> {
    let params = [
        ("stuid", stu_id),
        ("year", year),
        ("month", month),
        ("type", typ),
    ];
    let spider_res: SpiderCardHistory =
        spider_data("/pt/card/history", &params).await?;
    Ok(spider_res)
}
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderEmail {
    pub unReadCount: Option<u32>,
}
pub async fn get_email(stu_id: &str) -> AppResult<SpiderEmail> {
    let params = [("stuid", stu_id)];
    let spider_res: SpiderEmail =
        spider_data("/pt/email", &params).await?;
    Ok(spider_res)
}
